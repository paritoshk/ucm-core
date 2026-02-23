//! Confidence scoring module — transforms raw dependency data into actionable intelligence.
//!
//! Implements three approaches from the research synthesis:
//! 1. Noisy-OR fusion (Google Knowledge Vault, KDD 2014) — for multi-source evidence
//! 2. Temporal decay (TempValid framework, ACL 2024) — confidence is not timeless
//! 3. Bayesian update — for incorporating new evidence
//!
//! A static dependency graph says "A depends on B."
//! A probabilistic one says "A depends on B with 0.87 confidence based on
//! static analysis (0.92) and test coverage (0.75), decaying at 0.01/day
//! since last verification."

use serde::{Deserialize, Serialize};

/// Noisy-OR model for combining independent evidence sources.
///
/// P(edge) = 1 - Π(1 - P(source_i))
///
/// Better than simple multiplication for redundant paths — when multiple
/// independent sources confirm a dependency, confidence compounds rather
/// than multiplies pessimistically.
///
/// Reference: Google Knowledge Vault (KDD 2014)
pub fn noisy_or(confidences: &[f64]) -> f64 {
    if confidences.is_empty() {
        return 0.0;
    }
    1.0 - confidences.iter().map(|c| 1.0 - c.clamp(0.0, 1.0)).product::<f64>()
}

/// Temporal confidence decay using exponential model.
///
/// confidence(t) = base_confidence × exp(-λ × days_since_verification)
///
/// Decay rates vary by dependency type:
/// - Import statements:  λ ≈ 0.001  (very slow, imports rarely become invalid)
/// - Call graph edges:   λ ≈ 0.005  (slow, call relationships are stable)
/// - Test coverage:      λ ≈ 0.01   (moderate, tests may become stale)
/// - Heuristic inferences: λ ≈ 0.05 (moderate-fast)
/// - API traffic patterns: λ ≈ 0.1  (fast, traffic changes frequently)
///
/// Reference: TempValid framework (ACL 2024)
pub fn temporal_decay(base_confidence: f64, lambda: f64, days_since_verification: f64) -> f64 {
    let decay_factor = (-lambda * days_since_verification).exp();
    (base_confidence * decay_factor).clamp(0.0, 1.0)
}

/// Simple Bayesian update: combine prior belief with new evidence.
///
/// P(H|E) = P(E|H) × P(H) / P(E)
///
/// Simplified: uses likelihood ratio to update prior.
pub fn bayesian_update(prior: f64, likelihood_ratio: f64) -> f64 {
    let odds = prior / (1.0 - prior + f64::EPSILON);
    let posterior_odds = odds * likelihood_ratio;
    let posterior = posterior_odds / (1.0 + posterior_odds);
    posterior.clamp(0.0, 1.0)
}

/// Propagate confidence through a transitive chain.
///
/// For a path A → B → C, the confidence of the indirect dependency A → C
/// depends on the approach:
///
/// - Simple multiplication: P(A→C) = P(A→B) × P(B→C)
///   Fast but pessimistic — each hop reduces confidence multiplicatively.
///
/// - Noisy-OR over all paths: better for redundant paths
///   P(A→C) = 1 - Π(1 - P(path_i)) where P(path_i) = product of edges
pub fn chain_confidence(edge_confidences: &[f64]) -> f64 {
    edge_confidences.iter().product()
}

/// Combined multi-path confidence: given multiple paths between A and C,
/// compute the overall confidence using noisy-OR over each path's
/// chain confidence.
pub fn multi_path_confidence(paths: &[Vec<f64>]) -> f64 {
    let path_confidences: Vec<f64> = paths.iter()
        .map(|path| chain_confidence(path))
        .collect();
    noisy_or(&path_confidences)
}

/// Confidence report for explaining to humans.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceReport {
    pub raw_score: f64,
    pub decayed_score: f64,
    pub tier: String,
    pub sources: Vec<SourceContribution>,
    pub days_since_verified: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceContribution {
    pub source: String,
    pub individual_confidence: f64,
    pub observed_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noisy_or_single_source() {
        assert!((noisy_or(&[0.8]) - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn test_noisy_or_compounds_agreement() {
        // Two sources at 0.8: 1 - (0.2 × 0.2) = 0.96
        let result = noisy_or(&[0.8, 0.8]);
        assert!((result - 0.96).abs() < 0.001);
    }

    #[test]
    fn test_noisy_or_empty() {
        assert_eq!(noisy_or(&[]), 0.0);
    }

    #[test]
    fn test_temporal_decay() {
        // After 0 days, no decay
        assert!((temporal_decay(0.95, 0.01, 0.0) - 0.95).abs() < 0.001);

        // After 30 days with λ=0.01: 0.95 × exp(-0.3) ≈ 0.703
        let decayed = temporal_decay(0.95, 0.01, 30.0);
        assert!(decayed < 0.95);
        assert!(decayed > 0.5);

        // After 365 days with λ=0.001 (import): 0.95 × exp(-0.365) ≈ 0.659
        // Imports decay very slowly
        let import_decay = temporal_decay(0.95, 0.001, 365.0);
        assert!(import_decay > 0.6);
    }

    #[test]
    fn test_bayesian_update() {
        // Prior 0.5 with strong confirming evidence (LR=10)
        let posterior = bayesian_update(0.5, 10.0);
        assert!(posterior > 0.9);

        // Prior 0.5 with disconfirming evidence (LR=0.1)
        let posterior = bayesian_update(0.5, 0.1);
        assert!(posterior < 0.15);
    }

    #[test]
    fn test_chain_confidence() {
        // 3-hop chain: 0.95 × 0.90 × 0.80 = 0.684
        let chain = chain_confidence(&[0.95, 0.90, 0.80]);
        assert!((chain - 0.684).abs() < 0.001);
    }

    #[test]
    fn test_multi_path() {
        // Two paths from A to C:
        // Path 1: 0.9 × 0.8 = 0.72
        // Path 2: 0.7 × 0.6 = 0.42
        // Noisy-OR: 1 - (1-0.72)(1-0.42) = 1 - 0.28×0.58 = 0.8376
        let result = multi_path_confidence(&[
            vec![0.9, 0.8],
            vec![0.7, 0.6],
        ]);
        assert!((result - 0.8376).abs() < 0.001);
    }
}
