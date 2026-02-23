//! Edge model for the context graph with Bayesian confidence scoring.
//!
//! Each edge carries a confidence score that represents our belief in the
//! relationship's existence and current validity. Confidence is computed via
//! multi-source Bayesian fusion and decays over time using temporal decay
//! (TempValid framework, ACL 2024).
//!
//! Design decisions:
//! - Noisy-OR for multi-path confidence (Google Knowledge Vault, KDD 2014)
//! - Temporal decay rates vary by discovery source
//! - Each edge tracks its full evidence provenance chain

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::entity::DiscoverySource;

/// The type of relationship between two entities.
///
/// Inspired by Code Property Graph (Yamaguchi et al., IEEE S&P 2014) —
/// layered overlays for code structure, control flow, data flow, and
/// QA-specific relationships.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationType {
    /// A imports/uses B (static dependency)
    Imports,
    /// A calls B (call graph edge)
    Calls,
    /// A is tested by B (test coverage mapping)
    TestedBy,
    /// A implements requirement B
    Implements,
    /// A depends on B (general dependency)
    DependsOn,
    /// A is required by B (requirement traceability)
    RequiredBy,
    /// A contains B (structural containment: module→function)
    Contains,
    /// A extends/inherits B
    Extends,
    /// A reads/writes same data as B (data flow)
    DataFlow,
    /// A was changed together with B historically (co-change)
    CoChanged,
}

/// An edge in the context graph with confidence scoring and provenance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UcmEdge {
    /// What kind of relationship this represents
    pub relation_type: RelationType,

    /// Fused confidence score in [0.0, 1.0]
    /// Updated via Bayesian fusion from multiple evidence sources
    pub confidence: f64,

    /// Individual evidence sources that contributed to this edge
    pub evidence: Vec<EvidenceSource>,

    /// When this edge was first discovered
    pub discovered_at: DateTime<Utc>,

    /// When this edge was last verified (by test, trace, or re-analysis)
    pub verified_at: Option<DateTime<Utc>>,

    /// Decay rate (lambda) for temporal confidence decay
    /// - Import statements: λ ≈ 0.001 (very slow decay)
    /// - Call graph edges: λ ≈ 0.005 (slow decay)
    /// - Heuristic inferences: λ ≈ 0.05 (moderate decay)
    /// - API traffic patterns: λ ≈ 0.1 (fast decay)
    pub decay_rate: f64,
}

/// A single piece of evidence supporting an edge's existence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceSource {
    /// What discovered this evidence
    pub source: DiscoverySource,
    /// Confidence from this individual source [0.0, 1.0]
    pub confidence: f64,
    /// When this evidence was collected
    pub observed_at: DateTime<Utc>,
    /// Human-readable description of the evidence
    pub description: String,
}

impl UcmEdge {
    /// Create a new edge with a single initial evidence source.
    pub fn new(
        relation_type: RelationType,
        source: DiscoverySource,
        confidence: f64,
        description: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        let decay_rate = Self::default_decay_rate(&relation_type, &source);

        Self {
            relation_type,
            confidence,
            evidence: vec![EvidenceSource {
                source,
                confidence,
                observed_at: now,
                description: description.into(),
            }],
            discovered_at: now,
            verified_at: Some(now),
            decay_rate,
        }
    }

    /// Add new evidence and re-fuse confidence using Bayesian update.
    ///
    /// When sources agree, confidence compounds; when they conflict,
    /// the system expresses uncertainty rather than picking a winner.
    pub fn add_evidence(&mut self, source: DiscoverySource, confidence: f64, description: impl Into<String>) {
        self.evidence.push(EvidenceSource {
            source,
            confidence,
            observed_at: Utc::now(),
            description: description.into(),
        });

        // Re-fuse all evidence using noisy-OR model:
        // P(edge) = 1 - Π(1 - P(source_i))
        // This compounds agreement and expresses uncertainty on conflict.
        self.confidence = crate::confidence::noisy_or(
            &self.evidence.iter().map(|e| e.confidence).collect::<Vec<_>>()
        );

        self.verified_at = Some(Utc::now());
    }

    /// Get the current confidence after applying temporal decay.
    ///
    /// confidence(t) = base_confidence × exp(-λ × days_since_verification)
    /// Reference: TempValid framework (ACL 2024)
    pub fn decayed_confidence(&self) -> f64 {
        let last_verified = self.verified_at.unwrap_or(self.discovered_at);
        let days_elapsed = (Utc::now() - last_verified).num_hours() as f64 / 24.0;
        crate::confidence::temporal_decay(self.confidence, self.decay_rate, days_elapsed)
    }

    /// Mark this edge as recently verified (resets decay timer).
    pub fn verify(&mut self) {
        self.verified_at = Some(Utc::now());
    }

    /// Default decay rate based on relation type and discovery source.
    fn default_decay_rate(relation_type: &RelationType, source: &DiscoverySource) -> f64 {
        match (relation_type, source) {
            // Import statements from static analysis — very stable
            (RelationType::Imports, DiscoverySource::StaticAnalysis) => 0.001,
            // Call graph from static analysis — stable
            (RelationType::Calls, DiscoverySource::StaticAnalysis) => 0.005,
            // Test coverage — moderately stable
            (RelationType::TestedBy, _) => 0.01,
            // Requirement links — slow decay
            (RelationType::Implements | RelationType::RequiredBy, _) => 0.008,
            // Co-change from git history — moderate decay
            (RelationType::CoChanged, _) => 0.03,
            // API traffic patterns — fast decay
            (_, DiscoverySource::ApiTraffic) => 0.1,
            // Heuristic / historical — moderate-fast decay
            (_, DiscoverySource::HistoricalContext) => 0.05,
            // Default
            _ => 0.02,
        }
    }
}

/// Display tier for confidence scores.
/// Dual representation: continuous internally, three-tier for UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfidenceTier {
    /// ≥ 0.85 — high confidence, solid display
    High,
    /// 0.60 - 0.84 — medium confidence, translucent display
    Medium,
    /// < 0.60 — low confidence, dashed display
    Low,
}

impl ConfidenceTier {
    pub fn from_score(confidence: f64) -> Self {
        if confidence >= 0.85 {
            Self::High
        } else if confidence >= 0.60 {
            Self::Medium
        } else {
            Self::Low
        }
    }

    pub fn emoji(&self) -> &str {
        match self {
            Self::High => "🟢",
            Self::Medium => "🟡",
            Self::Low => "🔴",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_creation() {
        let edge = UcmEdge::new(
            RelationType::Imports,
            DiscoverySource::StaticAnalysis,
            0.95,
            "import statement found in AST",
        );
        assert_eq!(edge.relation_type, RelationType::Imports);
        assert!((edge.confidence - 0.95).abs() < f64::EPSILON);
        assert_eq!(edge.evidence.len(), 1);
    }

    #[test]
    fn test_evidence_fusion() {
        let mut edge = UcmEdge::new(
            RelationType::DependsOn,
            DiscoverySource::StaticAnalysis,
            0.80,
            "static analysis found dependency",
        );
        edge.add_evidence(
            DiscoverySource::ApiTraffic,
            0.70,
            "API traffic confirms runtime dependency",
        );
        // Noisy-OR: 1 - (1-0.8)(1-0.7) = 1 - 0.2*0.3 = 0.94
        assert!((edge.confidence - 0.94).abs() < 0.01);
        assert_eq!(edge.evidence.len(), 2);
    }

    #[test]
    fn test_confidence_tier() {
        assert_eq!(ConfidenceTier::from_score(0.90), ConfidenceTier::High);
        assert_eq!(ConfidenceTier::from_score(0.72), ConfidenceTier::Medium);
        assert_eq!(ConfidenceTier::from_score(0.45), ConfidenceTier::Low);
    }
}
