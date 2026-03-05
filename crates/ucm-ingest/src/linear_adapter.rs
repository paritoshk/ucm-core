//! Linear adapter — converts Linear issues into Requirement/Feature entities.

use serde::{Deserialize, Serialize};
use ucm_graph_core::entity::*;
use ucm_graph_core::event::*;

/// A Linear issue structure matching the GraphQL API response shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearIssue {
    pub identifier: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub state: String,
    #[serde(default)]
    pub priority: String,
    #[serde(default)]
    pub labels: Vec<String>,
    pub assignee: Option<String>,
    pub url: Option<String>,
}

/// Parse a Linear issue into context events.
pub fn ingest_linear_issue(issue: &LinearIssue) -> Vec<UcmEvent> {
    let mut events = Vec::new();

    // Create Requirement entity
    let entity_id = EntityId::local(&format!("linear/{}", issue.identifier), &issue.identifier);
    events.push(UcmEvent::new(EventPayload::EntityDiscovered {
        entity_id: entity_id.clone(),
        kind: EntityKind::Requirement {
            ticket_id: Some(issue.identifier.clone()),
            acceptance_criteria: issue.labels.clone(),
        },
        name: format!("{}: {}", issue.identifier, issue.title),
        file_path: format!("linear/{}", issue.identifier),
        language: "requirement".to_string(),
        source: DiscoverySource::TicketSystem,
        line_range: None,
    }));

    // Create Feature entity from the issue
    events.push(UcmEvent::new(EventPayload::EntityDiscovered {
        entity_id: EntityId::local(
            &format!("linear/{}", issue.identifier),
            &format!("feature:{}", issue.identifier),
        ),
        kind: EntityKind::Feature {
            description: issue.description.clone(),
            source: "linear".to_string(),
        },
        name: issue.title.clone(),
        file_path: format!("linear/{}", issue.identifier),
        language: "requirement".to_string(),
        source: DiscoverySource::TicketSystem,
        line_range: None,
    }));

    events
}

/// Parse multiple Linear issues from JSON.
pub fn ingest_linear_issues_json(json: &str) -> Result<Vec<UcmEvent>, serde_json::Error> {
    let issues: Vec<LinearIssue> = serde_json::from_str(json)?;
    let mut events = Vec::new();
    for issue in &issues {
        events.extend(ingest_linear_issue(issue));
    }
    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ingest_linear_issue() {
        let issue = LinearIssue {
            identifier: "ENG-123".into(),
            title: "Add user authentication".into(),
            description: "Implement OAuth2 login flow".into(),
            state: "In Progress".into(),
            priority: "High".into(),
            labels: vec!["auth".into(), "security".into()],
            assignee: Some("alice".into()),
            url: Some("https://linear.app/team/issue/ENG-123".into()),
        };

        let events = ingest_linear_issue(&issue);
        assert_eq!(
            events.len(),
            2,
            "Should create requirement + feature entities"
        );

        // Verify requirement entity
        match &events[0].payload {
            EventPayload::EntityDiscovered { name, source, .. } => {
                assert!(name.contains("ENG-123"));
                assert!(matches!(source, DiscoverySource::TicketSystem));
            }
            _ => panic!("First event should be EntityDiscovered"),
        }
    }
}
