//! Impact analysis — determines what is directly/indirectly impacted by a change.
//!
//! This module orchestrates the graph's impact_bfs with change classification
//! to produce a structured ImpactReport with explanations.
//!
//! References:
//! - Google TAP: reverse dependency traversal in build graph
//! - Meta PTS: MinDist (shortest path in dependency graph) is the most
//!   predictive feature for test relevance
//! - Test failure likelihood diminishes beyond MinDist=10

use crate::explanation::{explain_impact, explain_not_impacted, ExplanationChain};
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use ucm_graph_core::edge::ConfidenceTier;
use ucm_graph_core::entity::EntityId;
use ucm_graph_core::graph::{ImpactType, ImpactedEntity, NotImpactedEntity, UcmGraph};

/// Full impact report for a change set — the primary output of the reasoning engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactReport {
    /// What entities were changed
    pub changes: Vec<ChangeDescription>,
    /// Entities directly impacted (1-hop dependency)
    pub direct_impacts: Vec<ImpactEntry>,
    /// Entities indirectly impacted (2+ hop, confidence-weighted)
    pub indirect_impacts: Vec<ImpactEntry>,
    /// Entities determined to NOT be impacted (with explanation)
    pub not_impacted: Vec<NotImpactedEntry>,
    /// Ambiguities and conflicts detected
    pub ambiguities: Vec<AmbiguityEntry>,
    /// Graph traversal statistics
    pub stats: ImpactStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeDescription {
    pub entity_id: String,
    pub name: String,
    pub change_type: String,
    pub file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactEntry {
    pub entity_id: String,
    pub name: String,
    pub confidence: f64,
    pub tier: String,
    pub depth: usize,
    pub path: Vec<String>,
    pub reason: String,
    pub explanation_chain: ExplanationChain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotImpactedEntry {
    pub entity_id: String,
    pub name: String,
    pub confidence: f64,
    pub reason: String,
    pub explanation_chain: ExplanationChain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbiguityEntry {
    pub entity_id: Option<String>,
    pub ambiguity_type: String,
    pub description: String,
    pub sources: Vec<String>,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactStats {
    pub total_entities: usize,
    pub directly_impacted: usize,
    pub indirectly_impacted: usize,
    pub not_impacted: usize,
    pub max_depth_reached: usize,
}

/// Reverse BFS from a set of changed entities.
/// Returns all transitively affected entities with confidence scores.
///
/// This implements the dependency-based test selection approach used by
/// Google TAP: reverse BFS/DFS from changed nodes in the build graph.
/// Confidence decays along the path: each hop reduces confidence multiplicatively.
pub fn impact_bfs(
    graph: &UcmGraph,
    changed: &[EntityId],
    min_confidence: f64,
    max_depth: usize,
) -> Vec<ImpactedEntity> {
    let inner = graph.inner();
    let mut visited: HashMap<petgraph::stable_graph::NodeIndex, ImpactedEntity> = HashMap::new();
    let mut queue: VecDeque<(petgraph::stable_graph::NodeIndex, f64, usize, Vec<String>)> =
        VecDeque::new();

    // Seed with changed entities
    for id in changed {
        if let Some(idx) = graph.entity_node_index(id) {
            let entity = inner.node_weight(idx).unwrap();
            visited.insert(
                idx,
                ImpactedEntity {
                    entity_id: id.clone(),
                    name: entity.name.clone(),
                    confidence: 1.0,
                    depth: 0,
                    impact_type: ImpactType::Direct,
                    path: vec![id.as_str().to_string()],
                    reason: "Directly changed".to_string(),
                },
            );
            queue.push_back((idx, 1.0, 0, vec![id.as_str().to_string()]));
        }
    }

    while let Some((current, current_confidence, depth, path)) = queue.pop_front() {
        if depth >= max_depth {
            continue;
        }

        for edge in inner.edges_directed(current, Direction::Incoming) {
            let neighbor = edge.source();
            let edge_weight = edge.weight();

            let propagated = current_confidence * edge_weight.decayed_confidence();
            if propagated < min_confidence {
                continue;
            }

            let neighbor_entity = match inner.node_weight(neighbor) {
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

    let changed_indices: HashSet<_> = changed
        .iter()
        .filter_map(|id| graph.entity_node_index(id))
        .collect();

    visited
        .into_iter()
        .filter(|(idx, _)| !changed_indices.contains(idx))
        .map(|(_, impact)| impact)
        .collect()
}

/// Find entities that are NOT impacted by a change set.
pub fn find_not_impacted(
    graph: &UcmGraph,
    changed: &[EntityId],
    impacted: &[ImpactedEntity],
) -> Vec<NotImpactedEntity> {
    let inner = graph.inner();
    let changed_set: HashSet<&str> = changed.iter().map(|id| id.as_str()).collect();
    let impacted_set: HashSet<&str> = impacted.iter().map(|i| i.entity_id.as_str()).collect();

    inner
        .node_weights()
        .filter(|entity| {
            !changed_set.contains(entity.id.as_str()) && !impacted_set.contains(entity.id.as_str())
        })
        .map(|entity| {
            let has_path = has_path_to_any(graph, &entity.id, changed);
            let reason = if has_path {
                "Path exists but confidence below threshold".to_string()
            } else {
                "No graph path exists to changed entities".to_string()
            };
            let confidence = if has_path { 0.60 } else { 0.90 };
            NotImpactedEntity {
                entity_id: entity.id.clone(),
                name: entity.name.clone(),
                confidence,
                reason,
            }
        })
        .collect()
}

fn has_path_to_any(graph: &UcmGraph, from: &EntityId, targets: &[EntityId]) -> bool {
    let inner = graph.inner();
    let from_idx = match graph.entity_node_index(from) {
        Some(idx) => idx,
        None => return false,
    };
    let target_indices: HashSet<_> = targets
        .iter()
        .filter_map(|id| graph.entity_node_index(id))
        .collect();

    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(from_idx);

    while let Some(current) = queue.pop_front() {
        if target_indices.contains(&current) {
            return true;
        }
        if !visited.insert(current) {
            continue;
        }
        for neighbor in inner.neighbors_directed(current, Direction::Outgoing) {
            queue.push_back(neighbor);
        }
    }
    false
}

/// Analyze the impact of a set of changes on the context graph.
///
/// This is the core reasoning function. It:
/// 1. Identifies changed entities
/// 2. Runs reverse BFS to find impacted entities (with confidence decay)
/// 3. Identifies NOT-impacted entities (with explanations)
/// 4. Generates explanation chains for each conclusion
pub fn analyze_impact(
    graph: &UcmGraph,
    changed_entities: &[EntityId],
    min_confidence: f64,
    max_depth: usize,
) -> ImpactReport {
    // Run impact BFS
    let impacted = impact_bfs(graph, changed_entities, min_confidence, max_depth);
    let not_impacted_entities = find_not_impacted(graph, changed_entities, &impacted);

    // Classify into direct and indirect
    let mut direct_impacts = Vec::new();
    let mut indirect_impacts = Vec::new();
    let mut max_depth_reached: usize = 0;

    for impact in &impacted {
        let tier = ConfidenceTier::from_score(impact.confidence);
        let explanation = explain_impact(&impact.name, &impact.path, impact.confidence);

        let entry = ImpactEntry {
            entity_id: impact.entity_id.as_str().to_string(),
            name: impact.name.clone(),
            confidence: impact.confidence,
            tier: format!("{} {:?}", tier.emoji(), tier),
            depth: impact.depth,
            path: impact.path.clone(),
            reason: impact.reason.clone(),
            explanation_chain: explanation,
        };

        max_depth_reached = max_depth_reached.max(impact.depth);

        match impact.impact_type {
            ImpactType::Direct => direct_impacts.push(entry),
            ImpactType::Indirect => indirect_impacts.push(entry),
        }
    }

    // Build not-impacted entries with explanations
    let not_impacted: Vec<NotImpactedEntry> = not_impacted_entities
        .iter()
        .map(|ni| {
            let explanation = explain_not_impacted(&ni.name, &ni.reason, ni.confidence);
            NotImpactedEntry {
                entity_id: ni.entity_id.as_str().to_string(),
                name: ni.name.clone(),
                confidence: ni.confidence,
                reason: ni.reason.clone(),
                explanation_chain: explanation,
            }
        })
        .collect();

    // Build change descriptions
    let changes: Vec<ChangeDescription> = changed_entities
        .iter()
        .map(|id| {
            let entity = graph.get_entity(id);
            ChangeDescription {
                entity_id: id.as_str().to_string(),
                name: entity
                    .map(|e| e.name.clone())
                    .unwrap_or_else(|| "Unknown".into()),
                change_type: "Modified".into(),
                file_path: entity.map(|e| e.file_path.clone()).unwrap_or_default(),
            }
        })
        .collect();

    let stats = ImpactStats {
        total_entities: graph.stats().entity_count,
        directly_impacted: direct_impacts.len(),
        indirectly_impacted: indirect_impacts.len(),
        not_impacted: not_impacted.len(),
        max_depth_reached,
    };

    ImpactReport {
        changes,
        direct_impacts,
        indirect_impacts,
        not_impacted,
        ambiguities: Vec::new(), // Filled by ambiguity detector
        stats,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ucm_graph_core::edge::*;
    use ucm_graph_core::entity::*;

    fn build_test_graph() -> UcmGraph {
        let mut graph = UcmGraph::new();

        let entities = vec![
            ("src/auth/service.ts", "validateToken", "validateToken"),
            ("src/api/middleware.ts", "authMiddleware", "authMiddleware"),
            (
                "src/payments/checkout.ts",
                "processPayment",
                "processPayment",
            ),
            ("src/admin/reports.ts", "generateReport", "generateReport"),
        ];

        for (file, symbol, name) in &entities {
            graph
                .add_entity(UcmEntity::new(
                    EntityId::local(file, symbol),
                    EntityKind::Function {
                        is_async: true,
                        parameter_count: 1,
                        return_type: None,
                    },
                    *name,
                    *file,
                    "typescript",
                    DiscoverySource::StaticAnalysis,
                ))
                .unwrap();
        }

        // middleware → validateToken
        graph
            .add_relationship(
                &EntityId::local("src/api/middleware.ts", "authMiddleware"),
                &EntityId::local("src/auth/service.ts", "validateToken"),
                UcmEdge::new(
                    RelationType::Imports,
                    DiscoverySource::StaticAnalysis,
                    0.95,
                    "imports directly",
                ),
            )
            .unwrap();

        // processPayment → middleware
        graph
            .add_relationship(
                &EntityId::local("src/payments/checkout.ts", "processPayment"),
                &EntityId::local("src/api/middleware.ts", "authMiddleware"),
                UcmEdge::new(
                    RelationType::DependsOn,
                    DiscoverySource::StaticAnalysis,
                    0.80,
                    "uses auth middleware",
                ),
            )
            .unwrap();

        graph
    }

    #[test]
    fn test_impact_analysis() {
        let graph = build_test_graph();
        let changed = vec![EntityId::local("src/auth/service.ts", "validateToken")];

        let report = analyze_impact(&graph, &changed, 0.1, 10);

        // Should have direct impacts
        assert!(
            !report.direct_impacts.is_empty(),
            "Should have direct impacts"
        );
        assert!(report
            .direct_impacts
            .iter()
            .any(|i| i.name == "authMiddleware"));

        // Should have indirect impacts
        assert!(
            !report.indirect_impacts.is_empty(),
            "Should have indirect impacts"
        );
        assert!(report
            .indirect_impacts
            .iter()
            .any(|i| i.name == "processPayment"));

        // Should have not-impacted
        assert!(!report.not_impacted.is_empty(), "Should have not-impacted");
        assert!(report
            .not_impacted
            .iter()
            .any(|n| n.name == "generateReport"));

        // All entries should have explanation chains
        for impact in &report.direct_impacts {
            assert!(!impact.explanation_chain.steps.is_empty());
        }
    }

    #[test]
    fn test_impact_report_serializable() {
        let graph = build_test_graph();
        let changed = vec![EntityId::local("src/auth/service.ts", "validateToken")];
        let report = analyze_impact(&graph, &changed, 0.1, 10);

        let json = serde_json::to_string_pretty(&report).unwrap();
        assert!(json.contains("explanation_chain"));
        assert!(json.contains("not_impacted"));

        // Round-trip
        let _: ImpactReport = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn test_find_not_impacted() {
        let graph = build_test_graph();
        // validateToken is changed
        let changed = vec![EntityId::local("src/auth/service.ts", "validateToken")];
        // authMiddleware is impacted (directly)
        let impacted = vec![ImpactedEntity {
            entity_id: EntityId::local("src/api/middleware.ts", "authMiddleware"),
            name: "authMiddleware".to_string(),
            confidence: 0.95,
            depth: 1,
            impact_type: ImpactType::Direct,
            path: vec!["validateToken".to_string(), "authMiddleware".to_string()],
            reason: "imports directly".to_string(),
        }];

        let not_impacted = find_not_impacted(&graph, &changed, &impacted);

        // generateReport should be in not_impacted because it has no path to validateToken
        let report_ni = not_impacted.iter().find(|ni| ni.name == "generateReport");
        assert!(report_ni.is_some());
        assert_eq!(
            report_ni.unwrap().reason,
            "No graph path exists to changed entities"
        );
        assert_eq!(report_ni.unwrap().confidence, 0.90);

        // processPayment should be in not_impacted because it WAS NOT in the impacted list passed in,
        // even though it HAS a path to validateToken in the graph.
        let payment_ni = not_impacted.iter().find(|ni| ni.name == "processPayment");
        assert!(payment_ni.is_some());
        assert_eq!(
            payment_ni.unwrap().reason,
            "Path exists but confidence below threshold"
        );
        assert_eq!(payment_ni.unwrap().confidence, 0.60);

        // validateToken itself should NOT be in not_impacted
        assert!(!not_impacted.iter().any(|ni| ni.name == "validateToken"));

        // authMiddleware itself should NOT be in not_impacted
        assert!(!not_impacted.iter().any(|ni| ni.name == "authMiddleware"));
    }
}
