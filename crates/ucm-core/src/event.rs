//! Event model — append-only mutation log for the context graph.
//!
//! Every mutation to the context is an event. The event log is the single
//! source of truth — the graph is always rebuildable from events.
//!
//! Design: Datomic-inspired immutable datom model where each fact is
//! (Entity, Attribute, Value, Transaction, Added/Retracted), mapped to
//! our domain as typed context mutation events.
//!
//! Schema evolution uses additive-only changes plus upcasting.
//! Reference: Event Sourcing best practices (Microsoft, Greg Young)

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::entity::{EntityId, EntityKind, DiscoverySource};
use crate::edge::RelationType;

/// A mutation event in the context graph's append-only log.
///
/// Every field change, relationship addition, or confidence update
/// is captured as a typed event with full provenance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UcmEvent {
    /// Unique event identifier (UUID v7 for time-ordering)
    pub event_id: Uuid,
    /// When this event occurred
    pub timestamp: DateTime<Utc>,
    /// Which event caused this one (for causation chains)
    pub causation_id: Option<Uuid>,
    /// Schema version for upcasting support
    pub schema_version: u32,
    /// The actual mutation
    pub payload: EventPayload,
}

/// The typed payload of a context event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventPayload {
    /// A new entity was discovered in the codebase
    EntityDiscovered {
        entity_id: EntityId,
        kind: EntityKind,
        name: String,
        file_path: String,
        language: String,
        source: DiscoverySource,
        line_range: Option<(usize, usize)>,
    },

    /// An entity was removed (file deleted, function removed, etc.)
    EntityRemoved {
        entity_id: EntityId,
        reason: String,
    },

    /// A relationship between entities was discovered
    DependencyLinked {
        source_entity: EntityId,
        target_entity: EntityId,
        relation_type: RelationType,
        confidence: f64,
        source: DiscoverySource,
        description: String,
    },

    /// New evidence was added to an existing edge
    ConfidenceUpdated {
        source_entity: EntityId,
        target_entity: EntityId,
        new_evidence_confidence: f64,
        source: DiscoverySource,
        description: String,
    },

    /// A code change was detected
    ChangeDetected {
        file_path: String,
        change_type: ChangeType,
        affected_entities: Vec<EntityId>,
        before_snapshot: Option<String>,
        after_snapshot: Option<String>,
    },

    /// A conflict or ambiguity was flagged
    ConflictFlagged {
        entity_id: EntityId,
        conflict_type: ConflictType,
        sources: Vec<ConflictSource>,
        description: String,
    },

    /// An edge was verified (resets decay timer)
    EdgeVerified {
        source_entity: EntityId,
        target_entity: EntityId,
    },

    /// Batch ingestion completed
    IngestionCompleted {
        source: DiscoverySource,
        entities_count: usize,
        edges_count: usize,
        duration_ms: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    /// Function/method signature changed
    SignatureChange,
    /// Function/method body changed
    BodyChange,
    /// New entity added
    EntityAdded,
    /// Entity deleted
    EntityDeleted,
    /// File renamed/moved
    FileRenamed { old_path: String },
    /// Import added/removed
    ImportChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    /// Requirements say X but code does Y
    RequirementDrift,
    /// Two sources provide contradictory information
    SourceConflict,
    /// Expected data is missing
    MissingData,
    /// Coverage gap — code exists but no test covers it
    CoverageGap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictSource {
    pub source_type: String,
    pub claimed_value: String,
    pub confidence: f64,
}

impl UcmEvent {
    /// Create a new event with auto-generated UUID v7 and current timestamp.
    pub fn new(payload: EventPayload) -> Self {
        Self {
            event_id: Uuid::now_v7(),
            timestamp: Utc::now(),
            causation_id: None,
            schema_version: 1,
            payload,
        }
    }

    /// Create a new event with a causation chain link.
    pub fn caused_by(payload: EventPayload, parent: Uuid) -> Self {
        Self {
            event_id: Uuid::now_v7(),
            timestamp: Utc::now(),
            causation_id: Some(parent),
            schema_version: 1,
            payload,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = UcmEvent::new(EventPayload::EntityDiscovered {
            entity_id: EntityId::local("src/auth/service.ts", "validateToken"),
            kind: EntityKind::Function {
                is_async: true,
                parameter_count: 1,
                return_type: Some("boolean".into()),
            },
            name: "validateToken".into(),
            file_path: "src/auth/service.ts".into(),
            language: "typescript".into(),
            source: DiscoverySource::StaticAnalysis,
            line_range: Some((10, 25)),
        });
        assert_eq!(event.schema_version, 1);
        assert!(event.causation_id.is_none());
    }

    #[test]
    fn test_causation_chain() {
        let parent = UcmEvent::new(EventPayload::ChangeDetected {
            file_path: "src/auth/service.ts".into(),
            change_type: ChangeType::SignatureChange,
            affected_entities: vec![EntityId::local("src/auth/service.ts", "validateToken")],
            before_snapshot: None,
            after_snapshot: None,
        });

        let child = UcmEvent::caused_by(
            EventPayload::ConfidenceUpdated {
                source_entity: EntityId::local("src/auth/service.ts", "validateToken"),
                target_entity: EntityId::local("src/api/middleware.ts", "authMiddleware"),
                new_evidence_confidence: 0.95,
                source: DiscoverySource::StaticAnalysis,
                description: "re-analyzed after change".into(),
            },
            parent.event_id,
        );

        assert_eq!(child.causation_id, Some(parent.event_id));
    }

    #[test]
    fn test_event_serialization() {
        let event = UcmEvent::new(EventPayload::DependencyLinked {
            source_entity: EntityId::local("src/auth/service.ts", "AuthService"),
            target_entity: EntityId::local("src/db/client.ts", "DatabaseClient"),
            relation_type: RelationType::DependsOn,
            confidence: 0.92,
            source: DiscoverySource::StaticAnalysis,
            description: "import statement found".into(),
        });

        let json = serde_json::to_string_pretty(&event).unwrap();
        assert!(json.contains("DependencyLinked"));
        assert!(json.contains("0.92"));

        // Round-trip
        let deserialized: UcmEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.event_id, event.event_id);
    }
}
