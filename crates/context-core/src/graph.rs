//! Materialized context graph with Glean-style immutable fact layering.
//!
//! The graph is a petgraph StableGraph (stable indices across mutations)
//! wrapped with typed operations, ownership tracking, and query methods.
//!
//! Key design decisions:
//! - StableGraph for stable NodeIndex/EdgeIndex across removals (petgraph docs)
//! - Glean-style layers: each "fact layer" is an overlay that can add or hide
//!   facts from layers below. When a file changes, all facts owned by that file
//!   are hidden, and only affected files are re-indexed. O(changes) cost.
//! - SCIP identity: entities identified by globally-unique strings, so files
//!   can be re-indexed independently without graph-local ID coordination.
//!
//! References:
//! - petgraph StableGraph: https://docs.rs/petgraph
//! - Meta Glean fact stacking: https://glean.software/docs/angle/incrementality
//! - Sourcegraph SCIP: https://github.com/sourcegraph/scip

use std::collections::{HashMap, HashSet};
use petgraph::stable_graph::{StableGraph, NodeIndex, EdgeIndex};
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use serde::{Deserialize, Serialize};

use crate::entity::{ContextEntity, EntityId};
use crate::edge::{ContextEdge, ConfidenceTier, RelationType};

use crate::error::{ContextError, Result};

/// The materialized context graph — primary queryable data structure.
///
/// Wraps petgraph's `StableGraph` with typed operations and
/// SCIP-identity → NodeIndex lookup.
#[derive(Debug)]
pub struct ContextGraph {
    /// The underlying petgraph stable graph
    graph: StableGraph<ContextEntity, ContextEdge>,
    /// Fast lookup: SCIP EntityId string → NodeIndex
    entity_index: HashMap<String, NodeIndex>,
    /// Ownership tracking: which source file "owns" which entities
    /// Used for Glean-style incremental updates — when a file changes,
    /// all entities owned by that file are invalidated.
    ownership: HashMap<String, HashSet<NodeIndex>>,
}

impl ContextGraph {
    pub fn new() -> Self {
        Self {
            graph: StableGraph::new(),
            entity_index: HashMap::new(),
            ownership: HashMap::new(),
        }
    }

    // ─── Mutation Operations ───────────────────────────────────────

    /// Add an entity to the graph. Returns the node index.
    /// If an entity with the same SCIP ID exists, returns error.
    pub fn add_entity(&mut self, entity: ContextEntity) -> Result<NodeIndex> {
        let id_str = entity.id.as_str().to_string();
        if self.entity_index.contains_key(&id_str) {
            return Err(ContextError::DuplicateEntity(id_str));
        }

        let file_path = entity.file_path.clone();
        let idx = self.graph.add_node(entity);
        self.entity_index.insert(id_str, idx);

        // Track ownership: this file owns this entity
        self.ownership
            .entry(file_path)
            .or_default()
            .insert(idx);

        Ok(idx)
    }

    /// Add or update an entity (upsert semantics for replay safety).
    pub fn upsert_entity(&mut self, entity: ContextEntity) -> NodeIndex {
        let id_str = entity.id.as_str().to_string();
        if let Some(&idx) = self.entity_index.get(&id_str) {
            // Update existing
            if let Some(node) = self.graph.node_weight_mut(idx) {
                *node = entity;
            }
            idx
        } else {
            self.add_entity(entity).unwrap()
        }
    }

    /// Add a relationship between two entities.
    pub fn add_relationship(
        &mut self,
        from: &EntityId,
        to: &EntityId,
        edge: ContextEdge,
    ) -> Result<EdgeIndex> {
        let from_idx = self.resolve_entity(from)?;
        let to_idx = self.resolve_entity(to)?;
        Ok(self.graph.add_edge(from_idx, to_idx, edge))
    }

    /// Remove all entities and edges owned by a file path.
    /// This is the Glean-style "hide facts" operation for incremental updates.
    pub fn invalidate_file(&mut self, file_path: &str) -> Vec<EntityId> {
        let mut removed = Vec::new();

        if let Some(nodes) = self.ownership.remove(file_path) {
            for idx in nodes {
                if let Some(entity) = self.graph.remove_node(idx) {
                    self.entity_index.remove(entity.id.as_str());
                    removed.push(entity.id);
                }
            }
        }

        removed
    }

    // ─── Query Operations ──────────────────────────────────────────

    /// Get an entity by its SCIP ID.
    pub fn get_entity(&self, id: &EntityId) -> Option<&ContextEntity> {
        let idx = self.entity_index.get(id.as_str())?;
        self.graph.node_weight(*idx)
    }

    /// Get all entities in the graph.
    pub fn all_entities(&self) -> Vec<&ContextEntity> {
        self.graph.node_weights().collect()
    }

    /// Get direct dependencies of an entity (outgoing edges).
    pub fn dependencies(&self, id: &EntityId) -> Result<Vec<(&ContextEntity, &ContextEdge)>> {
        let idx = self.resolve_entity(id)?;
        Ok(self.graph
            .edges_directed(idx, Direction::Outgoing)
            .filter_map(|edge| {
                let target = self.graph.node_weight(edge.target())?;
                Some((target, edge.weight()))
            })
            .collect())
    }

    /// Get reverse dependencies of an entity (incoming edges).
    /// "What depends on this entity?"
    ///
    /// This is the core query for impact analysis — when entity X changes,
    /// which entities are affected? (Google TAP, Meta PTS)
    pub fn reverse_deps(&self, id: &EntityId) -> Result<Vec<(&ContextEntity, &ContextEdge)>> {
        let idx = self.resolve_entity(id)?;
        Ok(self.graph
            .edges_directed(idx, Direction::Incoming)
            .filter_map(|edge| {
                let source = self.graph.node_weight(edge.source())?;
                Some((source, edge.weight()))
            })
            .collect())
    }

    /// Reverse BFS from a set of changed entities.
    /// Returns all transitively affected entities with confidence scores.
    ///
    /// This implements the dependency-based test selection approach used by
    /// Google TAP: reverse BFS/DFS from changed nodes in the build graph.
    /// O(V+E) per change set.
    ///
    /// Confidence decays along the path: each hop reduces confidence
    /// multiplicatively (chain confidence).
    pub fn impact_bfs(
        &self,
        changed: &[EntityId],
        min_confidence: f64,
        max_depth: usize,
    ) -> Vec<ImpactedEntity> {
        let mut visited: HashMap<NodeIndex, ImpactedEntity> = HashMap::new();
        let mut queue: std::collections::VecDeque<(NodeIndex, f64, usize, Vec<String>)> =
            std::collections::VecDeque::new();

        // Seed with changed entities
        for id in changed {
            if let Ok(idx) = self.resolve_entity(id) {
                let entity = self.graph.node_weight(idx).unwrap();
                visited.insert(idx, ImpactedEntity {
                    entity_id: id.clone(),
                    name: entity.name.clone(),
                    confidence: 1.0, // Direct change = certainty
                    depth: 0,
                    impact_type: ImpactType::Direct,
                    path: vec![id.as_str().to_string()],
                    reason: "Directly changed".to_string(),
                });
                queue.push_back((idx, 1.0, 0, vec![id.as_str().to_string()]));
            }
        }

        // BFS with confidence-weighted propagation
        while let Some((current, current_confidence, depth, path)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }

            // Walk reverse edges (what depends on current?)
            for edge in self.graph.edges_directed(current, Direction::Incoming) {
                let neighbor = edge.source();
                let edge_weight = edge.weight();

                // Compute propagated confidence
                let propagated = current_confidence * edge_weight.decayed_confidence();
                if propagated < min_confidence {
                    continue; // Below threshold, prune
                }

                let neighbor_entity = match self.graph.node_weight(neighbor) {
                    Some(e) => e,
                    None => continue,
                };

                let mut new_path = path.clone();
                new_path.push(neighbor_entity.id.as_str().to_string());

                let impact = ImpactedEntity {
                    entity_id: neighbor_entity.id.clone(),
                    name: neighbor_entity.name.clone(),
                    confidence: propagated,
                    depth: depth + 1,
                    impact_type: if depth == 0 {
                        ImpactType::Direct
                    } else {
                        ImpactType::Indirect
                    },
                    path: new_path.clone(),
                    reason: format!(
                        "{} via {} ({})",
                        edge_weight.relation_type_str(),
                        path.last().unwrap_or(&"?".to_string()),
                        ConfidenceTier::from_score(propagated).emoji()
                    ),
                };

                // Only update if we found a higher-confidence path
                let should_update = match visited.get(&neighbor) {
                    Some(existing) => propagated > existing.confidence,
                    None => true,
                };

                if should_update {
                    visited.insert(neighbor, impact);
                    queue.push_back((neighbor, propagated, depth + 1, new_path));
                }
            }
        }

        // Remove the initially changed entities from the impact list
        let changed_indices: HashSet<_> = changed.iter()
            .filter_map(|id| self.entity_index.get(id.as_str()))
            .collect();

        visited.into_iter()
            .filter(|(idx, _)| !changed_indices.contains(idx))
            .map(|(_, impact)| impact)
            .collect()
    }

    /// Find entities that are NOT impacted by a change set.
    /// This is as important as finding impacted ones — explains WHY
    /// something doesn't need testing.
    pub fn not_impacted(
        &self,
        changed: &[EntityId],
        impacted: &[ImpactedEntity],
    ) -> Vec<NotImpactedEntity> {
        let changed_set: HashSet<&str> = changed.iter().map(|id| id.as_str()).collect();
        let impacted_set: HashSet<&str> = impacted.iter().map(|i| i.entity_id.as_str()).collect();

        self.graph
            .node_weights()
            .filter(|entity| {
                !changed_set.contains(entity.id.as_str())
                    && !impacted_set.contains(entity.id.as_str())
            })
            .map(|entity| {
                let reason = if self.has_path_to_any(&entity.id, changed) {
                    "Path exists but confidence below threshold".to_string()
                } else {
                    "No graph path exists to changed entities".to_string()
                };
                NotImpactedEntity {
                    entity_id: entity.id.clone(),
                    name: entity.name.clone(),
                    confidence: self.separation_confidence(&entity.id, changed),
                    reason,
                }
            })
            .collect()
    }

    /// Get graph statistics.
    pub fn stats(&self) -> GraphStats {
        let edge_count = self.graph.edge_count();
        let avg_confidence = if edge_count > 0 {
            self.graph.edge_weights()
                .map(|e| e.confidence)
                .sum::<f64>() / edge_count as f64
        } else {
            0.0
        };

        GraphStats {
            entity_count: self.graph.node_count(),
            edge_count,
            avg_confidence,
            files_tracked: self.ownership.len(),
        }
    }

    // ─── Serialization ─────────────────────────────────────────────

    /// Serialize the graph to JSON.
    pub fn to_json(&self) -> Result<String> {
        let snapshot = GraphSnapshot {
            entities: self.graph.node_weights().cloned().collect(),
            edges: self.graph.edge_indices().filter_map(|idx| {
                let (source, target) = self.graph.edge_endpoints(idx)?;
                let source_entity = self.graph.node_weight(source)?;
                let target_entity = self.graph.node_weight(target)?;
                let edge = self.graph.edge_weight(idx)?;
                Some(EdgeSnapshot {
                    from: source_entity.id.clone(),
                    to: target_entity.id.clone(),
                    edge: edge.clone(),
                })
            }).collect(),
        };
        Ok(serde_json::to_string_pretty(&snapshot)?)
    }

    // ─── Internal helpers ──────────────────────────────────────────

    fn resolve_entity(&self, id: &EntityId) -> Result<NodeIndex> {
        self.entity_index
            .get(id.as_str())
            .copied()
            .ok_or_else(|| ContextError::EntityNotFound(id.as_str().to_string()))
    }

    fn has_path_to_any(&self, from: &EntityId, targets: &[EntityId]) -> bool {
        let from_idx = match self.resolve_entity(from) {
            Ok(idx) => idx,
            Err(_) => return false,
        };
        let target_indices: HashSet<_> = targets.iter()
            .filter_map(|id| self.entity_index.get(id.as_str()).copied())
            .collect();

        // BFS from `from` following outgoing edges
        let mut visited = HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(from_idx);

        while let Some(current) = queue.pop_front() {
            if target_indices.contains(&current) {
                return true;
            }
            if !visited.insert(current) {
                continue;
            }
            for neighbor in self.graph.neighbors_directed(current, Direction::Outgoing) {
                queue.push_back(neighbor);
            }
        }
        false
    }

    fn separation_confidence(&self, entity: &EntityId, changed: &[EntityId]) -> f64 {
        // If no path exists, high confidence that it's not impacted
        if !self.has_path_to_any(entity, changed) {
            0.90
        } else {
            // Path exists but confidence too low — moderate separation confidence
            0.60
        }
    }
}

impl Default for ContextGraph {
    fn default() -> Self {
        Self::new()
    }
}

// Helper method on ContextEdge for display
impl ContextEdge {
    pub fn relation_type_str(&self) -> &str {
        match &self.relation_type {
            RelationType::Imports => "imports",
            RelationType::Calls => "calls",
            RelationType::TestedBy => "tested by",
            RelationType::Implements => "implements",
            RelationType::DependsOn => "depends on",
            RelationType::RequiredBy => "required by",
            RelationType::Contains => "contains",
            RelationType::Extends => "extends",
            RelationType::DataFlow => "data flow",
            RelationType::CoChanged => "co-changed with",
        }
    }
}

// ─── Result Types ──────────────────────────────────────────────────

/// An entity identified as impacted by a change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactedEntity {
    pub entity_id: EntityId,
    pub name: String,
    pub confidence: f64,
    pub depth: usize,
    pub impact_type: ImpactType,
    pub path: Vec<String>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImpactType {
    /// Directly references the changed entity
    Direct,
    /// Transitively depends on the changed entity
    Indirect,
}

/// An entity determined to NOT be impacted by a change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotImpactedEntity {
    pub entity_id: EntityId,
    pub name: String,
    pub confidence: f64,
    pub reason: String,
}

/// Serializable snapshot of the full graph.
#[derive(Debug, Serialize, Deserialize)]
struct GraphSnapshot {
    entities: Vec<ContextEntity>,
    edges: Vec<EdgeSnapshot>,
}

#[derive(Debug, Serialize, Deserialize)]
struct EdgeSnapshot {
    from: EntityId,
    to: EntityId,
    edge: ContextEdge,
}

/// Graph statistics summary.
#[derive(Debug, Serialize, Deserialize)]
pub struct GraphStats {
    pub entity_count: usize,
    pub edge_count: usize,
    pub avg_confidence: f64,
    pub files_tracked: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::*;
    use crate::edge::*;

    fn make_test_graph() -> ContextGraph {
        let mut graph = ContextGraph::new();

        // Create entities
        let auth_svc = ContextEntity::new(
            EntityId::local("src/auth/service.ts", "validateToken"),
            EntityKind::Function {
                is_async: true,
                parameter_count: 1,
                return_type: Some("boolean".into()),
            },
            "validateToken",
            "src/auth/service.ts",
            "typescript",
            DiscoverySource::StaticAnalysis,
        );

        let middleware = ContextEntity::new(
            EntityId::local("src/api/middleware.ts", "authMiddleware"),
            EntityKind::Function {
                is_async: true,
                parameter_count: 2,
                return_type: None,
            },
            "authMiddleware",
            "src/api/middleware.ts",
            "typescript",
            DiscoverySource::StaticAnalysis,
        );

        let payment = ContextEntity::new(
            EntityId::local("src/payments/checkout.ts", "processPayment"),
            EntityKind::Function {
                is_async: true,
                parameter_count: 1,
                return_type: Some("PaymentResult".into()),
            },
            "processPayment",
            "src/payments/checkout.ts",
            "typescript",
            DiscoverySource::StaticAnalysis,
        );

        let admin = ContextEntity::new(
            EntityId::local("src/admin/reports.ts", "generateReport"),
            EntityKind::Function {
                is_async: false,
                parameter_count: 0,
                return_type: Some("Report".into()),
            },
            "generateReport",
            "src/admin/reports.ts",
            "typescript",
            DiscoverySource::StaticAnalysis,
        );

        graph.add_entity(auth_svc).unwrap();
        graph.add_entity(middleware).unwrap();
        graph.add_entity(payment).unwrap();
        graph.add_entity(admin).unwrap();

        // middleware imports validateToken
        graph.add_relationship(
            &EntityId::local("src/api/middleware.ts", "authMiddleware"),
            &EntityId::local("src/auth/service.ts", "validateToken"),
            ContextEdge::new(
                RelationType::Imports,
                DiscoverySource::StaticAnalysis,
                0.95,
                "imports validateToken directly",
            ),
        ).unwrap();

        // payment depends on middleware (protected route)
        graph.add_relationship(
            &EntityId::local("src/payments/checkout.ts", "processPayment"),
            &EntityId::local("src/api/middleware.ts", "authMiddleware"),
            ContextEdge::new(
                RelationType::DependsOn,
                DiscoverySource::StaticAnalysis,
                0.80,
                "route uses authMiddleware",
            ),
        ).unwrap();

        // admin has NO connection to auth
        // (separate auth flow — this tests "not impacted" logic)

        graph
    }

    #[test]
    fn test_entity_lookup() {
        let graph = make_test_graph();
        let entity = graph.get_entity(&EntityId::local("src/auth/service.ts", "validateToken"));
        assert!(entity.is_some());
        assert_eq!(entity.unwrap().name, "validateToken");
    }

    #[test]
    fn test_duplicate_entity_error() {
        let mut graph = make_test_graph();
        let dup = ContextEntity::new(
            EntityId::local("src/auth/service.ts", "validateToken"),
            EntityKind::Function {
                is_async: false,
                parameter_count: 0,
                return_type: None,
            },
            "validateToken",
            "src/auth/service.ts",
            "typescript",
            DiscoverySource::StaticAnalysis,
        );
        assert!(graph.add_entity(dup).is_err());
    }

    #[test]
    fn test_reverse_deps() {
        let graph = make_test_graph();
        let rdeps = graph
            .reverse_deps(&EntityId::local("src/auth/service.ts", "validateToken"))
            .unwrap();
        assert_eq!(rdeps.len(), 1);
        assert_eq!(rdeps[0].0.name, "authMiddleware");
    }

    #[test]
    fn test_impact_bfs_direct() {
        let graph = make_test_graph();
        let changed = vec![EntityId::local("src/auth/service.ts", "validateToken")];
        let impacted = graph.impact_bfs(&changed, 0.1, 10);

        // authMiddleware should be directly impacted
        let auth_middleware = impacted.iter().find(|i| i.name == "authMiddleware");
        assert!(auth_middleware.is_some());
        assert_eq!(auth_middleware.unwrap().impact_type, ImpactType::Direct);
    }

    #[test]
    fn test_impact_bfs_indirect() {
        let graph = make_test_graph();
        let changed = vec![EntityId::local("src/auth/service.ts", "validateToken")];
        let impacted = graph.impact_bfs(&changed, 0.1, 10);

        // processPayment should be indirectly impacted
        let payment = impacted.iter().find(|i| i.name == "processPayment");
        assert!(payment.is_some());
        assert_eq!(payment.unwrap().impact_type, ImpactType::Indirect);
        // Confidence should be lower due to path length
        assert!(payment.unwrap().confidence < 0.95);
    }

    #[test]
    fn test_not_impacted() {
        let graph = make_test_graph();
        let changed = vec![EntityId::local("src/auth/service.ts", "validateToken")];
        let impacted = graph.impact_bfs(&changed, 0.1, 10);
        let not_impacted = graph.not_impacted(&changed, &impacted);

        // admin reports should NOT be impacted
        let admin = not_impacted.iter().find(|n| n.name == "generateReport");
        assert!(admin.is_some());
        assert!(admin.unwrap().reason.contains("No graph path"));
    }

    #[test]
    fn test_file_invalidation() {
        let mut graph = make_test_graph();
        assert!(graph.get_entity(&EntityId::local("src/auth/service.ts", "validateToken")).is_some());

        let removed = graph.invalidate_file("src/auth/service.ts");
        assert_eq!(removed.len(), 1);
        assert!(graph.get_entity(&EntityId::local("src/auth/service.ts", "validateToken")).is_none());
    }

    #[test]
    fn test_graph_stats() {
        let graph = make_test_graph();
        let stats = graph.stats();
        assert_eq!(stats.entity_count, 4);
        assert_eq!(stats.edge_count, 2);
        assert!(stats.avg_confidence > 0.0);
    }

    #[test]
    fn test_graph_serialization() {
        let graph = make_test_graph();
        let json = graph.to_json().unwrap();
        assert!(json.contains("validateToken"));
        assert!(json.contains("authMiddleware"));
    }
}
