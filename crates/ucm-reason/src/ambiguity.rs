//! Ambiguity detection — flags conflicts, drift, and missing data.
//!
//! The system must explicitly deal with uncertainty:
//! - Conflicting requirements (Jira says X, API logs show Y)
//! - Drift between tickets and reality
//! - Missing or partial data

use crate::impact::ImpactReport;
use serde::{Deserialize, Serialize};
use ucm_graph_core::edge::ConfidenceTier;
use ucm_graph_core::graph::UcmGraph;

/// Ambiguity report — flags found in the context graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbiguityReport {
    pub flags: Vec<AmbiguityFlag>,
    pub total_low_confidence_edges: usize,
    pub total_stale_edges: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbiguityFlag {
    pub flag_type: AmbiguityType,
    pub entity_id: Option<String>,
    pub description: String,
    pub evidence: Vec<String>,
    pub recommendation: String,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AmbiguityType {
    /// Two sources disagree about a fact
    SourceConflict,
    /// Requirements don't match observed behavior
    RequirementDrift,
    /// Expected information is missing
    MissingData,
    /// Edge confidence below threshold
    LowConfidence,
    /// Data is stale (high temporal decay)
    StaleData,
}

/// Detect ambiguities in the context graph.
pub fn detect_ambiguities(graph: &UcmGraph, confidence_threshold: f64) -> AmbiguityReport {
    let mut flags = Vec::new();
    let mut low_confidence_count = 0;
    let mut stale_count = 0;

    // Check all entities for issues
    let entities = graph.all_entities();
    for entity in &entities {
        // Check edges for low confidence
        if let Ok(deps) = graph.dependencies(&entity.id) {
            for (dep, edge) in &deps {
                let tier = ConfidenceTier::from_score(edge.confidence);

                if edge.confidence < confidence_threshold {
                    low_confidence_count += 1;
                    flags.push(AmbiguityFlag {
                        flag_type: AmbiguityType::LowConfidence,
                        entity_id: Some(entity.id.as_str().to_string()),
                        description: format!(
                            "Low confidence ({:.0}%) on relationship: {} → {}",
                            edge.confidence * 100.0,
                            entity.name,
                            dep.name
                        ),
                        evidence: edge.evidence.iter()
                            .map(|e| format!("{}: {:.0}% ({})", e.description, e.confidence * 100.0, e.observed_at))
                            .collect(),
                        recommendation: format!(
                            "Verify the relationship between {} and {} — consider running tests or re-analyzing",
                            entity.name, dep.name
                        ),
                        severity: tier.emoji().to_string(),
                    });
                }

                // Check for temporal staleness (decayed confidence significantly lower)
                let decayed = edge.decayed_confidence();
                if decayed < edge.confidence * 0.8 {
                    stale_count += 1;
                    flags.push(AmbiguityFlag {
                        flag_type: AmbiguityType::StaleData,
                        entity_id: Some(entity.id.as_str().to_string()),
                        description: format!(
                            "Stale relationship: {} → {} (base {:.0}%, decayed to {:.0}%)",
                            entity.name,
                            dep.name,
                            edge.confidence * 100.0,
                            decayed * 100.0
                        ),
                        evidence: vec![format!(
                            "Last verified: {:?}, decay rate: {}",
                            edge.verified_at, edge.decay_rate
                        )],
                        recommendation: "Re-verify this relationship with fresh analysis".into(),
                        severity: ConfidenceTier::from_score(decayed).emoji().to_string(),
                    });
                }
            }
        }
    }

    AmbiguityReport {
        flags,
        total_low_confidence_edges: low_confidence_count,
        total_stale_edges: stale_count,
    }
}

/// Enrich an impact report with ambiguities.
pub fn enrich_with_ambiguities(
    report: &mut ImpactReport,
    graph: &UcmGraph,
    confidence_threshold: f64,
) {
    let ambiguity_report = detect_ambiguities(graph, confidence_threshold);

    for flag in ambiguity_report.flags {
        report.ambiguities.push(crate::impact::AmbiguityEntry {
            entity_id: flag.entity_id,
            ambiguity_type: format!("{:?}", flag.flag_type),
            description: flag.description,
            sources: flag.evidence,
            recommendation: flag.recommendation,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ucm_graph_core::edge::*;
    use ucm_graph_core::entity::*;

    #[test]
    fn test_detect_low_confidence() {
        let mut graph = UcmGraph::new();

        graph
            .add_entity(UcmEntity::new(
                EntityId::local("src/a.ts", "fnA"),
                EntityKind::Function {
                    is_async: false,
                    parameter_count: 0,
                    return_type: None,
                },
                "fnA",
                "src/a.ts",
                "typescript",
                DiscoverySource::StaticAnalysis,
            ))
            .unwrap();

        graph
            .add_entity(UcmEntity::new(
                EntityId::local("src/b.ts", "fnB"),
                EntityKind::Function {
                    is_async: false,
                    parameter_count: 0,
                    return_type: None,
                },
                "fnB",
                "src/b.ts",
                "typescript",
                DiscoverySource::StaticAnalysis,
            ))
            .unwrap();

        // Add a low-confidence edge
        graph
            .add_relationship(
                &EntityId::local("src/a.ts", "fnA"),
                &EntityId::local("src/b.ts", "fnB"),
                UcmEdge::new(
                    RelationType::DependsOn,
                    DiscoverySource::HistoricalContext,
                    0.40,
                    "weak heuristic",
                ),
            )
            .unwrap();

        let report = detect_ambiguities(&graph, 0.60);
        assert!(report.total_low_confidence_edges > 0);
        assert!(!report.flags.is_empty());
    }
}
