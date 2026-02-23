//! API log adapter — converts access logs into ApiEndpoint entities
//! with traffic-based confidence scoring.

use ucm_core::entity::*;
use ucm_core::event::*;
use serde::{Deserialize, Serialize};

/// A simplified API access log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiLogEntry {
    pub method: String,
    pub path: String,
    pub status_code: u16,
    pub response_time_ms: u64,
    pub handler: Option<String>,
    pub timestamp: String,
}

/// Aggregate API logs into context events.
///
/// Groups by (method, path), counts calls, and creates
/// ApiEndpoint entities with traffic-based confidence.
pub fn ingest_api_logs(logs: &[ApiLogEntry]) -> Vec<UcmEvent> {
    let mut events = Vec::new();

    // Group by method + path
    let mut groups: std::collections::HashMap<String, Vec<&ApiLogEntry>> =
        std::collections::HashMap::new();
    for log in logs {
        let key = format!("{}:{}", log.method, log.path);
        groups.entry(key).or_default().push(log);
    }

    for (key, entries) in &groups {
        let first = entries[0];
        let call_count = entries.len();
        let avg_response = entries.iter().map(|e| e.response_time_ms).sum::<u64>() / call_count as u64;
        let error_rate = entries.iter().filter(|e| e.status_code >= 400).count() as f64 / call_count as f64;

        // Traffic-based confidence: more calls = higher confidence this endpoint is real
        let confidence = (call_count as f64 / 100.0).min(0.95).max(0.3);

        events.push(UcmEvent::new(EventPayload::EntityDiscovered {
            entity_id: EntityId::local(&format!("api/{}", first.path), &key),
            kind: EntityKind::ApiEndpoint {
                method: first.method.clone(),
                route: first.path.clone(),
                handler: first.handler.clone().unwrap_or_else(|| "unknown".into()),
            },
            name: format!("{} {} ({} calls, {}ms avg)", first.method, first.path, call_count, avg_response),
            file_path: format!("api/{}", first.path),
            language: "api".to_string(),
            source: DiscoverySource::ApiTraffic,
            line_range: None,
        }));

        // If error rate is high, flag it
        if error_rate > 0.05 {
            events.push(UcmEvent::new(EventPayload::ConflictFlagged {
                entity_id: EntityId::local(&format!("api/{}", first.path), &key),
                conflict_type: ucm_core::event::ConflictType::RequirementDrift,
                sources: vec![ucm_core::event::ConflictSource {
                    source_type: "api-logs".into(),
                    claimed_value: format!("{:.1}% error rate", error_rate * 100.0),
                    confidence,
                }],
                description: format!(
                    "Endpoint {} {} has {:.1}% error rate over {} calls",
                    first.method, first.path, error_rate * 100.0, call_count
                ),
            }));
        }
    }

    events
}

/// Parse API logs from JSON.
pub fn ingest_api_logs_json(json: &str) -> Result<Vec<UcmEvent>, serde_json::Error> {
    let logs: Vec<ApiLogEntry> = serde_json::from_str(json)?;
    Ok(ingest_api_logs(&logs))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ingest_api_logs() {
        let logs = vec![
            ApiLogEntry {
                method: "POST".into(),
                path: "/api/v1/auth/login".into(),
                status_code: 200,
                response_time_ms: 150,
                handler: Some("handleLogin".into()),
                timestamp: "2024-01-15T10:00:00Z".into(),
            },
            ApiLogEntry {
                method: "POST".into(),
                path: "/api/v1/auth/login".into(),
                status_code: 200,
                response_time_ms: 120,
                handler: Some("handleLogin".into()),
                timestamp: "2024-01-15T10:01:00Z".into(),
            },
        ];

        let events = ingest_api_logs(&logs);
        assert!(!events.is_empty());

        // Should create at least one ApiEndpoint entity
        let endpoints: Vec<_> = events.iter().filter(|e| matches!(
            &e.payload,
            EventPayload::EntityDiscovered { kind: EntityKind::ApiEndpoint { .. }, .. }
        )).collect();
        assert_eq!(endpoints.len(), 1);
    }
}
