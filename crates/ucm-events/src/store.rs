//! In-memory event store with the same API as a production RocksDB-backed store.
//!
//! Supports: append, replay, query by ID, stream by source,
//! and checkpoint-based resume.

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

use ucm_graph_core::event::{EventPayload, UcmEvent};

/// Append-only in-memory event store.
///
/// In production, this would be backed by RocksDB with column families
/// per stream. The API surface is identical — swap backend, not logic.
pub struct EventStore {
    /// All events in insertion order
    events: Vec<UcmEvent>,
    /// Index: event_id → position in events vec
    id_index: HashMap<Uuid, usize>,
    /// Index: stream_id (file path) → event positions
    stream_index: HashMap<String, Vec<usize>>,
    /// Last processed position for checkpoint/resume
    checkpoint: usize,
}

impl EventStore {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            id_index: HashMap::new(),
            stream_index: HashMap::new(),
            checkpoint: 0,
        }
    }

    /// Append an event to the log. Events are immutable once appended.
    pub fn append(&mut self, event: UcmEvent) {
        let pos = self.events.len();
        self.id_index.insert(event.event_id, pos);

        // Index by stream (extract file path from event payload)
        if let Some(stream_key) = Self::extract_stream_key(&event.payload) {
            self.stream_index.entry(stream_key).or_default().push(pos);
        }

        self.events.push(event);
    }

    /// Append a batch of events atomically.
    pub fn append_batch(&mut self, events: Vec<UcmEvent>) {
        for event in events {
            self.append(event);
        }
    }

    /// Get an event by its UUID.
    pub fn get_by_id(&self, id: &Uuid) -> Option<&UcmEvent> {
        self.id_index.get(id).and_then(|&pos| self.events.get(pos))
    }

    /// Replay all events from the beginning (or from a timestamp).
    pub fn replay(&self, from: Option<DateTime<Utc>>) -> Vec<&UcmEvent> {
        match from {
            Some(timestamp) => self
                .events
                .iter()
                .filter(|e| e.timestamp >= timestamp)
                .collect(),
            None => self.events.iter().collect(),
        }
    }

    /// Get events since the last checkpoint (for incremental projection).
    pub fn events_since_checkpoint(&self) -> &[UcmEvent] {
        &self.events[self.checkpoint..]
    }

    /// Advance the checkpoint to the current position.
    pub fn advance_checkpoint(&mut self) {
        self.checkpoint = self.events.len();
    }

    /// Get all events for a specific file/stream.
    pub fn stream(&self, stream_key: &str) -> Vec<&UcmEvent> {
        self.stream_index
            .get(stream_key)
            .map(|positions| {
                positions
                    .iter()
                    .filter_map(|&pos| self.events.get(pos))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Total number of events in the store.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if store is empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Get the causation chain for an event (trace back to root cause).
    pub fn causation_chain(&self, event_id: &Uuid) -> Vec<&UcmEvent> {
        let mut chain = Vec::new();
        let mut current_id = Some(*event_id);

        while let Some(id) = current_id {
            if let Some(event) = self.get_by_id(&id) {
                chain.push(event);
                current_id = event.causation_id;
            } else {
                break;
            }
        }

        chain.reverse(); // Root cause first
        chain
    }

    /// Extract a stream key from an event payload (typically the file path).
    fn extract_stream_key(payload: &EventPayload) -> Option<String> {
        match payload {
            EventPayload::EntityDiscovered { file_path, .. } => Some(file_path.clone()),
            EventPayload::ChangeDetected { file_path, .. } => Some(file_path.clone()),
            EventPayload::EntityRemoved { entity_id, .. } => {
                entity_id.file_path().map(|s| s.to_string())
            }
            _ => None,
        }
    }
}

impl Default for EventStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ucm_graph_core::entity::*;
    use ucm_graph_core::event::*;

    #[test]
    fn test_append_and_retrieve() {
        let mut store = EventStore::new();
        let event = UcmEvent::new(EventPayload::EntityDiscovered {
            entity_id: EntityId::local("src/main.rs", "main"),
            kind: EntityKind::Function {
                is_async: false,
                parameter_count: 0,
                return_type: None,
            },
            name: "main".into(),
            file_path: "src/main.rs".into(),
            language: "rust".into(),
            source: DiscoverySource::StaticAnalysis,
            line_range: Some((1, 10)),
        });

        let event_id = event.event_id;
        store.append(event);

        assert_eq!(store.len(), 1);
        assert!(store.get_by_id(&event_id).is_some());
    }

    #[test]
    fn test_replay() {
        let mut store = EventStore::new();
        for i in 0..5 {
            store.append(UcmEvent::new(EventPayload::EntityDiscovered {
                entity_id: EntityId::local("src/main.rs", &format!("fn_{i}")),
                kind: EntityKind::Function {
                    is_async: false,
                    parameter_count: 0,
                    return_type: None,
                },
                name: format!("fn_{i}"),
                file_path: "src/main.rs".into(),
                language: "rust".into(),
                source: DiscoverySource::StaticAnalysis,
                line_range: None,
            }));
        }

        let all = store.replay(None);
        assert_eq!(all.len(), 5);
    }

    #[test]
    fn test_checkpoint() {
        let mut store = EventStore::new();

        // Add 3 events
        for i in 0..3 {
            store.append(UcmEvent::new(EventPayload::EntityDiscovered {
                entity_id: EntityId::local("src/main.rs", &format!("fn_{i}")),
                kind: EntityKind::Function {
                    is_async: false,
                    parameter_count: 0,
                    return_type: None,
                },
                name: format!("fn_{i}"),
                file_path: "src/main.rs".into(),
                language: "rust".into(),
                source: DiscoverySource::StaticAnalysis,
                line_range: None,
            }));
        }

        store.advance_checkpoint();

        // Add 2 more events
        for i in 3..5 {
            store.append(UcmEvent::new(EventPayload::EntityDiscovered {
                entity_id: EntityId::local("src/main.rs", &format!("fn_{i}")),
                kind: EntityKind::Function {
                    is_async: false,
                    parameter_count: 0,
                    return_type: None,
                },
                name: format!("fn_{i}"),
                file_path: "src/main.rs".into(),
                language: "rust".into(),
                source: DiscoverySource::StaticAnalysis,
                line_range: None,
            }));
        }

        // Should only see the 2 new events
        assert_eq!(store.events_since_checkpoint().len(), 2);
    }

    #[test]
    fn test_causation_chain() {
        let mut store = EventStore::new();

        let root = UcmEvent::new(EventPayload::ChangeDetected {
            file_path: "src/auth/service.ts".into(),
            change_type: ChangeType::SignatureChange,
            affected_entities: vec![],
            before_snapshot: None,
            after_snapshot: None,
        });
        let root_id = root.event_id;
        store.append(root);

        let child = UcmEvent::caused_by(
            EventPayload::ConfidenceUpdated {
                source_entity: EntityId::local("src/auth/service.ts", "validateToken"),
                target_entity: EntityId::local("src/api/middleware.ts", "authMiddleware"),
                new_evidence_confidence: 0.9,
                source: DiscoverySource::StaticAnalysis,
                description: "re-analyzed".into(),
            },
            root_id,
        );
        let child_id = child.event_id;
        store.append(child);

        let chain = store.causation_chain(&child_id);
        assert_eq!(chain.len(), 2);
        assert_eq!(chain[0].event_id, root_id);
    }
}
