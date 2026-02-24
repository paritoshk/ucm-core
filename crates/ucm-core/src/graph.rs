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

use crate::entity::{UcmEntity, EntityId};
use crate::edge::{UcmEdge, RelationType};

use crate::error::{UcmError, Result};

/// The materialized context graph — primary queryable data structure.
///
/// Wraps petgraph's `StableGraph` with typed operations and
/// SCIP-identity → NodeIndex lookup.
#[derive(Debug)]
pub struct UcmGraph {
    /// The underlying petgraph stable graph
    graph: StableGraph<UcmEntity, UcmEdge>,
    /// Fast lookup: SCIP EntityId string → NodeIndex
    entity_index: HashMap<String, NodeIndex>,
    /// Ownership tracking: which source file "owns" which entities
    /// Used for Glean-style incremental updates — when a file changes,
    /// all entities owned by that file are invalidated.
    ownership: HashMap<String, HashSet<NodeIndex>>,
}

impl UcmGraph {
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
    pub fn add_entity(&mut self, entity: UcmEntity) -> Result<NodeIndex> {
        let id_str = entity.id.as_str().to_string();
        if self.entity_index.contains_key(&id_str) {
            return Err(UcmError::DuplicateEntity(id_str));
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
    pub fn upsert_entity(&mut self, entity: UcmEntity) -> NodeIndex {
        let id_str = entity.id.as_str().to_string();
        if let Some(&idx) = self.entity_index.get(&id_str) {
            // Update existing
            if let Some(node) = self.graph.node_weight_mut(idx) {
                *node = entity;
            }
            idx
        } else {
            // add_entity only fails if duplicate — we checked above, so this is safe.
            // Use expect with a clear message rather than unwrap.
            self.add_entity(entity).expect("add_entity: duplicate despite index miss (logic error)")
        }
    }

    /// Add a relationship between two entities.
    pub fn add_relationship(
        &mut self,
        from: &EntityId,
        to: &EntityId,
        edge: UcmEdge,
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
    pub fn get_entity(&self, id: &EntityId) -> Option<&UcmEntity> {
        let idx = self.entity_index.get(id.as_str())?;
        self.graph.node_weight(*idx)
    }

    /// Get all entities in the graph.
    pub fn all_entities(&self) -> Vec<&UcmEntity> {
        self.graph.node_weights().collect()
    }

    /// Get direct dependencies of an entity (outgoing edges).
    pub fn dependencies(&self, id: &EntityId) -> Result<Vec<(&UcmEntity, &UcmEdge)>> {
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
    pub fn reverse_deps(&self, id: &EntityId) -> Result<Vec<(&UcmEntity, &UcmEdge)>> {
        let idx = self.resolve_entity(id)?;
        Ok(self.graph
            .edges_directed(idx, Direction::Incoming)
            .filter_map(|edge| {
                let source = self.graph.node_weight(edge.source())?;
                Some((source, edge.weight()))
            })
            .collect())
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

    // ─── Low-Level Accessors (for external analysis modules) ──────

    /// Get a read-only reference to the inner petgraph.
    pub fn inner(&self) -> &StableGraph<UcmEntity, UcmEdge> {
        &self.graph
    }

    /// Resolve an EntityId to a NodeIndex, if it exists.
    pub fn entity_node_index(&self, id: &EntityId) -> Option<NodeIndex> {
        self.entity_index.get(id.as_str()).copied()
    }

    /// Get a reference to the full entity index map.
    pub fn entity_index_map(&self) -> &HashMap<String, NodeIndex> {
        &self.entity_index
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
            .ok_or_else(|| UcmError::EntityNotFound(id.as_str().to_string()))
    }

}

impl Default for UcmGraph {
    fn default() -> Self {
        Self::new()
    }
}

// Helper method on UcmEdge for display
impl UcmEdge {
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
    entities: Vec<UcmEntity>,
    edges: Vec<EdgeSnapshot>,
}

#[derive(Debug, Serialize, Deserialize)]
struct EdgeSnapshot {
    from: EntityId,
    to: EntityId,
    edge: UcmEdge,
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

    fn make_test_graph() -> UcmGraph {
        let mut graph = UcmGraph::new();

        // Create entities
        let auth_svc = UcmEntity::new(
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

        let middleware = UcmEntity::new(
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

        let payment = UcmEntity::new(
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

        let admin = UcmEntity::new(
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
            UcmEdge::new(
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
            UcmEdge::new(
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
        let dup = UcmEntity::new(
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
