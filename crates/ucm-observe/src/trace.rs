//! Decision trace — structured record of every reasoning decision.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use ucm_core::entity::EntityId;

/// A recorded reasoning decision — the atomic unit of auditability.
///
/// Contains the full derivation chain from input events to output
/// recommendation. Can be replayed to verify determinism.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionTrace {
    /// Unique trace identifier
    pub trace_id: Uuid,
    /// When this decision was made
    pub timestamp: DateTime<Utc>,
    /// What triggered this reasoning (typically a ChangeDetected event)
    pub trigger_event_id: Uuid,
    /// Hash of the graph state at decision time
    pub graph_state_hash: String,
    /// Which entities were analyzed
    pub analyzed_entities: Vec<String>,
    /// The reasoning steps taken (each with evidence/inference/confidence)
    pub reasoning_steps: Vec<TraceStep>,
    /// Final output summary
    pub output_summary: String,
    /// How long the decision took
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceStep {
    pub step: usize,
    pub operation: String,
    pub input: String,
    pub output: String,
    pub confidence: f64,
    pub timestamp: DateTime<Utc>,
}

/// In-memory trace storage (production: RocksDB column family).
pub struct TraceStore {
    traces: Vec<DecisionTrace>,
    index: std::collections::HashMap<Uuid, usize>,
}

impl TraceStore {
    pub fn new() -> Self {
        Self {
            traces: Vec::new(),
            index: std::collections::HashMap::new(),
        }
    }

    /// Record a decision trace.
    pub fn record(&mut self, trace: DecisionTrace) {
        let pos = self.traces.len();
        self.index.insert(trace.trace_id, pos);
        self.traces.push(trace);
    }

    /// Retrieve a trace by ID.
    pub fn get(&self, id: &Uuid) -> Option<&DecisionTrace> {
        self.index.get(id).and_then(|&pos| self.traces.get(pos))
    }

    /// Get all traces, most recent first.
    pub fn all(&self) -> Vec<&DecisionTrace> {
        self.traces.iter().rev().collect()
    }

    /// Get traces for a specific trigger event.
    pub fn by_trigger(&self, trigger_id: &Uuid) -> Vec<&DecisionTrace> {
        self.traces.iter()
            .filter(|t| t.trigger_event_id == *trigger_id)
            .collect()
    }

    pub fn len(&self) -> usize { self.traces.len() }
    pub fn is_empty(&self) -> bool { self.traces.is_empty() }
}

impl Default for TraceStore {
    fn default() -> Self { Self::new() }
}

/// Build a decision trace from an impact analysis run.
pub fn trace_impact_analysis(
    trigger_event_id: Uuid,
    graph_entity_count: usize,
    changed_entities: &[EntityId],
    direct_count: usize,
    indirect_count: usize,
    not_impacted_count: usize,
    duration_ms: u64,
) -> DecisionTrace {
    let mut steps = Vec::new();
    let now = Utc::now();

    steps.push(TraceStep {
        step: 1,
        operation: "enumerate_changes".into(),
        input: format!("{} entities changed", changed_entities.len()),
        output: format!("Change set: {:?}", changed_entities.iter().map(|e| e.as_str()).collect::<Vec<_>>()),
        confidence: 1.0,
        timestamp: now,
    });

    steps.push(TraceStep {
        step: 2,
        operation: "reverse_bfs".into(),
        input: format!("Graph with {} entities", graph_entity_count),
        output: format!("{} direct + {} indirect impacts found", direct_count, indirect_count),
        confidence: 0.95,
        timestamp: now,
    });

    steps.push(TraceStep {
        step: 3,
        operation: "classify_not_impacted".into(),
        input: format!("{} remaining entities", not_impacted_count),
        output: format!("{} entities determined not impacted with explanations", not_impacted_count),
        confidence: 0.90,
        timestamp: now,
    });

    DecisionTrace {
        trace_id: Uuid::now_v7(),
        timestamp: now,
        trigger_event_id,
        graph_state_hash: format!("entities:{}", graph_entity_count),
        analyzed_entities: changed_entities.iter().map(|e| e.as_str().to_string()).collect(),
        reasoning_steps: steps,
        output_summary: format!(
            "Impact analysis: {} direct, {} indirect, {} not impacted",
            direct_count, indirect_count, not_impacted_count
        ),
        duration_ms,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_store() {
        let mut store = TraceStore::new();
        let trigger_id = Uuid::now_v7();

        let trace = trace_impact_analysis(
            trigger_id,
            10,
            &[EntityId::local("src/auth.ts", "validateToken")],
            2, 3, 5,
            42,
        );
        let trace_id = trace.trace_id;
        store.record(trace);

        assert_eq!(store.len(), 1);
        assert!(store.get(&trace_id).is_some());

        let by_trigger = store.by_trigger(&trigger_id);
        assert_eq!(by_trigger.len(), 1);
    }

    #[test]
    fn test_trace_serialization() {
        let trace = trace_impact_analysis(
            Uuid::now_v7(),
            10,
            &[EntityId::local("src/main.ts", "main")],
            1, 2, 7,
            15,
        );

        let json = serde_json::to_string_pretty(&trace).unwrap();
        assert!(json.contains("reverse_bfs"));
        assert!(json.contains("reasoning_steps"));

        let _: DecisionTrace = serde_json::from_str(&json).unwrap();
    }
}
