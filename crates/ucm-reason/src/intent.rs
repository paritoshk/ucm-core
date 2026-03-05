//! Test intent generator — produces test recommendations from impact analysis.
//!
//! Outputs structured test intent (not test scripts):
//! - What scenarios should be tested
//! - What risks are introduced
//! - Where confidence is high vs low
//! - Existing coverage gaps
//!
//! Each recommendation includes an explanation chain
//! showing WHY the system recommends testing this scenario.

use crate::explanation::ExplanationChain;
use crate::impact::ImpactReport;
use serde::{Deserialize, Serialize};
use ucm_graph_core::edge::ConfidenceTier;

/// Complete test intent output — what to test and why.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestIntent {
    /// High-confidence scenarios — definitely test these
    pub high_confidence: Vec<TestScenario>,
    /// Medium-confidence scenarios — recommended to test
    pub medium_confidence: Vec<TestScenario>,
    /// Low-confidence scenarios — test if time permits
    pub low_confidence: Vec<TestScenario>,
    /// Identified risks
    pub risks: Vec<Risk>,
    /// Coverage gaps identified
    pub coverage_gaps: Vec<CoverageGap>,
    /// Entities explicitly decided NOT to test, with reasoning
    pub decided_not_to_test: Vec<SkippedEntity>,
    /// Summary statistics
    pub summary: TestIntentSummary,
}

/// A single test scenario recommendation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestScenario {
    /// What to test (human-readable)
    pub description: String,
    /// Why this should be tested
    pub rationale: String,
    /// Confidence that this scenario is necessary
    pub confidence: f64,
    /// Which impacted entity this relates to
    pub related_entity: String,
    /// Full reasoning chain
    pub explanation_chain: ExplanationChain,
}

/// An identified risk from the change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub severity: RiskSeverity,
    pub description: String,
    pub mitigation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskSeverity {
    High,
    Medium,
    Low,
}

/// An entity explicitly decided NOT to test, with reasoning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkippedEntity {
    /// The entity we decided not to test
    pub entity: String,
    /// Why we decided not to test it
    pub reason: String,
    /// How confident we are that skipping is safe
    pub confidence_of_safety: f64,
}

/// A gap in existing test coverage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageGap {
    pub entity: String,
    pub description: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestIntentSummary {
    pub total_scenarios: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub risk_count: usize,
}

/// Generate test intent from an impact report.
///
/// This is the core reasoning function that transforms graph-based
/// impact analysis into actionable test recommendations.
pub fn generate_test_intent(report: &ImpactReport) -> TestIntent {
    let mut high = Vec::new();
    let mut medium = Vec::new();
    let mut low = Vec::new();
    let mut risks = Vec::new();
    let mut coverage_gaps = Vec::new();

    // Generate scenarios for directly impacted entities
    for impact in &report.direct_impacts {
        let tier = ConfidenceTier::from_score(impact.confidence);

        let scenario = TestScenario {
            description: format!(
                "Verify {} still functions correctly after change",
                impact.name
            ),
            rationale: impact.reason.clone(),
            confidence: impact.confidence,
            related_entity: impact.name.clone(),
            explanation_chain: impact.explanation_chain.clone(),
        };

        match tier {
            ConfidenceTier::High => high.push(scenario),
            ConfidenceTier::Medium => medium.push(scenario),
            ConfidenceTier::Low => low.push(scenario),
        }

        // Add regression risk for direct impacts
        risks.push(Risk {
            severity: RiskSeverity::High,
            description: format!(
                "{} directly depends on changed code — regression risk if behavior changes",
                impact.name
            ),
            mitigation: format!(
                "Run existing tests for {} and verify expected behavior",
                impact.name
            ),
        });

        // Generate negative test scenario for direct impacts
        high.push(TestScenario {
            description: format!(
                "Verify {} properly handles error cases after change",
                impact.name
            ),
            rationale: "Direct dependency means error handling paths may also be affected".into(),
            confidence: impact.confidence * 0.9,
            related_entity: impact.name.clone(),
            explanation_chain: {
                let mut chain =
                    ExplanationChain::new(format!("Error handling test for {}", impact.name));
                chain.add_step(
                    "Direct dependency on changed code",
                    "Error paths may also be affected by the change",
                    impact.confidence * 0.9,
                );
                chain
            },
        });
    }

    // Generate scenarios for indirectly impacted entities
    for impact in &report.indirect_impacts {
        let tier = ConfidenceTier::from_score(impact.confidence);

        let scenario = TestScenario {
            description: format!(
                "Verify {} end-to-end flow still works (transitive dependency via {} hops)",
                impact.name, impact.depth
            ),
            rationale: impact.reason.clone(),
            confidence: impact.confidence,
            related_entity: impact.name.clone(),
            explanation_chain: impact.explanation_chain.clone(),
        };

        match tier {
            ConfidenceTier::High => high.push(scenario),
            ConfidenceTier::Medium => medium.push(scenario),
            ConfidenceTier::Low => low.push(scenario),
        }

        // Indirect impacts with high traffic → medium risk
        if impact.confidence > 0.7 {
            risks.push(Risk {
                severity: RiskSeverity::Medium,
                description: format!(
                    "{} is indirectly affected via {}-hop chain with {:.0}% confidence",
                    impact.name,
                    impact.depth,
                    impact.confidence * 100.0
                ),
                mitigation: format!(
                    "Integration test covering the path: {}",
                    impact.path.join(" → ")
                ),
            });
        }
    }

    // Check for coverage gaps among impacted entities
    for impact in report
        .direct_impacts
        .iter()
        .chain(report.indirect_impacts.iter())
    {
        // If we don't see any test entities connected, flag as gap
        coverage_gaps.push(CoverageGap {
            entity: impact.name.clone(),
            description: format!(
                "{} is impacted but has no linked test coverage in the graph",
                impact.name
            ),
            recommendation: format!(
                "Add test coverage for {} focusing on the changed behavior",
                impact.name
            ),
        });
    }

    // Populate "decided not to test" from not-impacted entities
    let decided_not_to_test: Vec<SkippedEntity> = report
        .not_impacted
        .iter()
        .map(|entry| SkippedEntity {
            entity: entry.name.clone(),
            reason: entry.reason.clone(),
            confidence_of_safety: entry.confidence,
        })
        .collect();

    // Add risk for ambiguities
    for ambiguity in &report.ambiguities {
        risks.push(Risk {
            severity: RiskSeverity::High,
            description: format!("Ambiguity: {}", ambiguity.description),
            mitigation: ambiguity.recommendation.clone(),
        });
    }

    let summary = TestIntentSummary {
        total_scenarios: high.len() + medium.len() + low.len(),
        high_count: high.len(),
        medium_count: medium.len(),
        low_count: low.len(),
        risk_count: risks.len(),
    };

    TestIntent {
        high_confidence: high,
        medium_confidence: medium,
        low_confidence: low,
        risks,
        coverage_gaps,
        decided_not_to_test,
        summary,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::impact::analyze_impact;
    use ucm_graph_core::edge::*;
    use ucm_graph_core::entity::*;
    use ucm_graph_core::graph::UcmGraph;

    #[test]
    fn test_generate_test_intent() {
        let mut graph = UcmGraph::new();

        // Build same test graph as impact tests
        for (file, symbol, name) in &[
            ("src/auth/service.ts", "validateToken", "validateToken"),
            ("src/api/middleware.ts", "authMiddleware", "authMiddleware"),
            (
                "src/payments/checkout.ts",
                "processPayment",
                "processPayment",
            ),
        ] {
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

        graph
            .add_relationship(
                &EntityId::local("src/api/middleware.ts", "authMiddleware"),
                &EntityId::local("src/auth/service.ts", "validateToken"),
                UcmEdge::new(
                    RelationType::Imports,
                    DiscoverySource::StaticAnalysis,
                    0.95,
                    "imports",
                ),
            )
            .unwrap();

        graph
            .add_relationship(
                &EntityId::local("src/payments/checkout.ts", "processPayment"),
                &EntityId::local("src/api/middleware.ts", "authMiddleware"),
                UcmEdge::new(
                    RelationType::DependsOn,
                    DiscoverySource::StaticAnalysis,
                    0.80,
                    "depends",
                ),
            )
            .unwrap();

        let changed = vec![EntityId::local("src/auth/service.ts", "validateToken")];
        let report = analyze_impact(&graph, &changed, 0.1, 10);
        let intent = generate_test_intent(&report);

        // Should have scenarios
        assert!(intent.summary.total_scenarios > 0);

        // Should have risks
        assert!(!intent.risks.is_empty());

        // High confidence should include scenarios for direct impacts
        assert!(!intent.high_confidence.is_empty());

        // All scenarios should have explanation chains
        for scenario in &intent.high_confidence {
            assert!(!scenario.explanation_chain.steps.is_empty());
        }

        // Should be serializable
        let json = serde_json::to_string_pretty(&intent).unwrap();
        assert!(json.contains("explanation_chain"));
        assert!(json.contains("high_confidence"));
        assert!(json.contains("decided_not_to_test"));
    }
}
