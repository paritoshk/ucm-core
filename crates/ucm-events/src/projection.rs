//! Event-to-graph projection — replay events to build/update the context graph.
//!
//! The projection pattern: each event type maps to a graph mutation using
//! MERGE/upsert semantics for replay safety. This means replaying the same
//! events produces the same graph — idempotent by design.
//!
//! Reference: Telicent CORE platform event-to-graph projection

use ucm_core::edge::UcmEdge;
use ucm_core::entity::UcmEntity;
use ucm_core::event::{EventPayload, UcmEvent};
use ucm_core::graph::UcmGraph;

/// Projects events from the event store into a materialized context graph.
pub struct GraphProjection;

impl GraphProjection {
    /// Replay a sequence of events to build a fresh graph from scratch.
    pub fn replay_all(events: &[UcmEvent]) -> UcmGraph {
        let mut graph = UcmGraph::new();
        for event in events {
            Self::apply_event(&mut graph, event);
        }
        graph
    }

    /// Apply a single event to an existing graph (incremental update).
    pub fn apply_event(graph: &mut UcmGraph, event: &UcmEvent) {
        match &event.payload {
            EventPayload::EntityDiscovered {
                entity_id,
                kind,
                name,
                file_path,
                language,
                source,
                line_range,
            } => {
                let mut entity = UcmEntity::new(
                    entity_id.clone(),
                    kind.clone(),
                    name.clone(),
                    file_path.clone(),
                    language.clone(),
                    source.clone(),
                );
                if let Some((start, end)) = line_range {
                    entity = entity.with_line_range(*start, *end);
                }
                graph.upsert_entity(entity);
            }

            EventPayload::EntityRemoved { entity_id, .. } => {
                // Remove by invalidating the entity's file
                if let Some(file_path) = entity_id.file_path() {
                    graph.invalidate_file(file_path);
                }
            }

            EventPayload::DependencyLinked {
                source_entity,
                target_entity,
                relation_type,
                confidence,
                source,
                description,
            } => {
                let edge = UcmEdge::new(
                    relation_type.clone(),
                    source.clone(),
                    *confidence,
                    description.clone(),
                );
                // Ignore errors if entities don't exist yet
                let _ = graph.add_relationship(source_entity, target_entity, edge);
            }

            EventPayload::ChangeDetected { file_path, .. } => {
                // Invalidate all entities owned by this file
                // (Glean-style: hide all facts owned by the changed file)
                graph.invalidate_file(file_path);
            }

            // Other event types don't mutate graph structure directly
            _ => {}
        }
    }

    /// Apply a batch of events incrementally.
    pub fn apply_batch(graph: &mut UcmGraph, events: &[UcmEvent]) {
        for event in events {
            Self::apply_event(graph, event);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ucm_core::edge::RelationType;
    use ucm_core::entity::*;

    #[test]
    fn test_replay_builds_graph() {
        let events = vec![
            UcmEvent::new(EventPayload::EntityDiscovered {
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
            }),
            UcmEvent::new(EventPayload::EntityDiscovered {
                entity_id: EntityId::local("src/api/middleware.ts", "authMiddleware"),
                kind: EntityKind::Function {
                    is_async: true,
                    parameter_count: 2,
                    return_type: None,
                },
                name: "authMiddleware".into(),
                file_path: "src/api/middleware.ts".into(),
                language: "typescript".into(),
                source: DiscoverySource::StaticAnalysis,
                line_range: None,
            }),
            UcmEvent::new(EventPayload::DependencyLinked {
                source_entity: EntityId::local("src/api/middleware.ts", "authMiddleware"),
                target_entity: EntityId::local("src/auth/service.ts", "validateToken"),
                relation_type: RelationType::Imports,
                confidence: 0.95,
                source: DiscoverySource::StaticAnalysis,
                description: "import statement".into(),
            }),
        ];

        let graph = GraphProjection::replay_all(&events);
        let stats = graph.stats();
        assert_eq!(stats.entity_count, 2);
        assert_eq!(stats.edge_count, 1);
    }

    #[test]
    fn test_replay_is_idempotent() {
        let events = vec![UcmEvent::new(EventPayload::EntityDiscovered {
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
            line_range: None,
        })];

        // Replaying same events twice should yield same graph
        let graph1 = GraphProjection::replay_all(&events);
        let graph2 = GraphProjection::replay_all(&events);
        assert_eq!(graph1.stats().entity_count, graph2.stats().entity_count);
    }
}
