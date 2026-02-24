//! Git history adapter — mines co-change patterns from version control
//! to produce `CoChanged` relationship edges with `HistoricalContext` discovery source.
//!
//! In production, this would parse `git log --name-only` output to identify
//! files that frequently change together, indicating implicit coupling.

use serde::{Deserialize, Serialize};
use ucm_core::edge::RelationType;
use ucm_core::entity::*;
use ucm_core::event::*;

/// A co-change entry mined from git history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoChangeEntry {
    /// First file in the co-change pair
    pub file_a: String,
    /// Second file in the co-change pair
    pub file_b: String,
    /// Number of commits where both files changed together
    pub co_change_count: u32,
    /// ISO timestamp of the most recent co-change
    pub last_seen: String,
}

/// Ingest co-change entries from git history into context events.
///
/// Each pair produces a `DependencyLinked` event with:
/// - `RelationType::CoChanged`
/// - `DiscoverySource::HistoricalContext`
/// - Confidence scaled by co-change frequency: `min(count / 50, 0.90).max(0.20)`
pub fn ingest_co_changes(entries: &[CoChangeEntry]) -> Vec<UcmEvent> {
    let mut events = Vec::new();

    for entry in entries {
        let confidence = (entry.co_change_count as f64 / 50.0).clamp(0.20, 0.90);

        // Create a module-level entity for each file if not already tracked
        let id_a = EntityId::local(&entry.file_a, &entry.file_a);
        let id_b = EntityId::local(&entry.file_b, &entry.file_b);

        events.push(UcmEvent::new(EventPayload::EntityDiscovered {
            entity_id: id_a.clone(),
            kind: EntityKind::Module {
                language: "unknown".into(),
                exports: vec![],
            },
            name: entry.file_a.clone(),
            file_path: entry.file_a.clone(),
            language: "unknown".into(),
            source: DiscoverySource::HistoricalContext,
            line_range: None,
        }));

        events.push(UcmEvent::new(EventPayload::EntityDiscovered {
            entity_id: id_b.clone(),
            kind: EntityKind::Module {
                language: "unknown".into(),
                exports: vec![],
            },
            name: entry.file_b.clone(),
            file_path: entry.file_b.clone(),
            language: "unknown".into(),
            source: DiscoverySource::HistoricalContext,
            line_range: None,
        }));

        events.push(UcmEvent::new(EventPayload::DependencyLinked {
            source_entity: id_a,
            target_entity: id_b,
            relation_type: RelationType::CoChanged,
            confidence,
            source: DiscoverySource::HistoricalContext,
            description: format!(
                "Co-changed {} times in git history (last seen: {})",
                entry.co_change_count, entry.last_seen
            ),
        }));
    }

    events
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ingest_co_changes() {
        let entries = vec![
            CoChangeEntry {
                file_a: "src/auth/service.ts".into(),
                file_b: "src/auth/middleware.ts".into(),
                co_change_count: 25,
                last_seen: "2024-12-01T10:00:00Z".into(),
            },
            CoChangeEntry {
                file_a: "src/payments/checkout.ts".into(),
                file_b: "src/payments/refund.ts".into(),
                co_change_count: 80,
                last_seen: "2025-01-15T14:30:00Z".into(),
            },
        ];

        let events = ingest_co_changes(&entries);

        // 3 events per entry: 2 entity discoveries + 1 dependency link
        assert_eq!(events.len(), 6);

        // Check confidence scaling
        // 25/50 = 0.50 (within bounds)
        // 80/50 = 1.60, clamped to 0.90
        let dep_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(&e.payload, EventPayload::DependencyLinked { .. }))
            .collect();
        assert_eq!(dep_events.len(), 2);

        if let EventPayload::DependencyLinked {
            confidence,
            relation_type,
            source,
            ..
        } = &dep_events[0].payload
        {
            assert!(*confidence > 0.49 && *confidence < 0.51);
            assert!(matches!(relation_type, RelationType::CoChanged));
            assert!(matches!(source, DiscoverySource::HistoricalContext));
        } else {
            panic!("Expected DependencyLinked");
        }

        if let EventPayload::DependencyLinked { confidence, .. } = &dep_events[1].payload {
            assert!((*confidence - 0.90).abs() < 0.01);
        } else {
            panic!("Expected DependencyLinked");
        }
    }
}
