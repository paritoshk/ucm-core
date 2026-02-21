//! Replay debugger — re-derive decisions from event log to verify determinism.
//!
//! Given a decision trace and the event log, the replay debugger can:
//! 1. Reproduce the graph state at the time of the decision
//! 2. Re-run the same reasoning
//! 3. Compare the result with the stored trace
//! 4. Report any divergences (non-determinism bugs)

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::trace::DecisionTrace;

/// Result of a replay verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayResult {
    pub original_trace_id: Uuid,
    pub replayed_trace_id: Uuid,
    pub is_deterministic: bool,
    pub divergences: Vec<Divergence>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Divergence {
    pub step: usize,
    pub field: String,
    pub original_value: String,
    pub replayed_value: String,
}

/// Compare two traces to find divergences.
pub fn compare_traces(original: &DecisionTrace, replayed: &DecisionTrace) -> ReplayResult {
    let mut divergences = Vec::new();

    // Compare step counts
    if original.reasoning_steps.len() != replayed.reasoning_steps.len() {
        divergences.push(Divergence {
            step: 0,
            field: "step_count".into(),
            original_value: original.reasoning_steps.len().to_string(),
            replayed_value: replayed.reasoning_steps.len().to_string(),
        });
    }

    // Compare individual steps
    let min_steps = original.reasoning_steps.len().min(replayed.reasoning_steps.len());
    for i in 0..min_steps {
        let orig = &original.reasoning_steps[i];
        let replay = &replayed.reasoning_steps[i];

        if orig.output != replay.output {
            divergences.push(Divergence {
                step: i + 1,
                field: "output".into(),
                original_value: orig.output.clone(),
                replayed_value: replay.output.clone(),
            });
        }

        if (orig.confidence - replay.confidence).abs() > 0.01 {
            divergences.push(Divergence {
                step: i + 1,
                field: "confidence".into(),
                original_value: format!("{:.4}", orig.confidence),
                replayed_value: format!("{:.4}", replay.confidence),
            });
        }
    }

    // Compare final output
    if original.output_summary != replayed.output_summary {
        divergences.push(Divergence {
            step: 0,
            field: "output_summary".into(),
            original_value: original.output_summary.clone(),
            replayed_value: replayed.output_summary.clone(),
        });
    }

    let is_deterministic = divergences.is_empty();
    let summary = if is_deterministic {
        "Replay verified: decision is deterministic".into()
    } else {
        format!("Replay found {} divergences", divergences.len())
    };

    ReplayResult {
        original_trace_id: original.trace_id,
        replayed_trace_id: replayed.trace_id,
        is_deterministic,
        divergences,
        summary,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trace::*;
    use ucm_core::entity::EntityId;

    #[test]
    fn test_deterministic_replay() {
        let trigger = Uuid::now_v7();
        let changed = vec![EntityId::local("src/auth.ts", "validate")];

        let trace1 = trace_impact_analysis(trigger, 10, &changed, 2, 3, 5, 42);
        let trace2 = trace_impact_analysis(trigger, 10, &changed, 2, 3, 5, 43);

        let result = compare_traces(&trace1, &trace2);
        assert!(result.is_deterministic);
    }

    #[test]
    fn test_divergent_replay() {
        let trigger = Uuid::now_v7();
        let changed = vec![EntityId::local("src/auth.ts", "validate")];

        let trace1 = trace_impact_analysis(trigger, 10, &changed, 2, 3, 5, 42);
        let trace2 = trace_impact_analysis(trigger, 10, &changed, 3, 4, 3, 43); // Different results

        let result = compare_traces(&trace1, &trace2);
        assert!(!result.is_deterministic);
        assert!(!result.divergences.is_empty());
    }
}
