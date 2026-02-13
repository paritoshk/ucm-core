//! Jira/ticket adapter — converts ticket JSON into Requirement entities.

use context_core::entity::*;
use context_core::edge::*;
use context_core::event::*;
use serde::{Deserialize, Serialize};

/// A simplified Jira ticket structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraTicket {
    pub key: String,
    pub summary: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    pub linked_files: Vec<String>,
    pub status: String,
}

/// Parse a Jira ticket into context events.
pub fn ingest_ticket(ticket: &JiraTicket) -> Vec<ContextEvent> {
    let mut events = Vec::new();

    // Create Requirement entity
    let entity_id = EntityId::local(&format!("jira/{}", ticket.key), &ticket.key);
    events.push(ContextEvent::new(EventPayload::EntityDiscovered {
        entity_id: entity_id.clone(),
        kind: EntityKind::Requirement {
            ticket_id: Some(ticket.key.clone()),
            acceptance_criteria: ticket.acceptance_criteria.clone(),
        },
        name: format!("{}: {}", ticket.key, ticket.summary),
        file_path: format!("jira/{}", ticket.key),
        language: "requirement".to_string(),
        source: DiscoverySource::TicketSystem,
        line_range: None,
    }));

    // Also create a Feature entity from the ticket
    events.push(ContextEvent::new(EventPayload::EntityDiscovered {
        entity_id: EntityId::local(&format!("jira/{}", ticket.key), &format!("feature:{}", ticket.key)),
        kind: EntityKind::Feature {
            description: ticket.description.clone(),
            source: "jira".to_string(),
        },
        name: ticket.summary.clone(),
        file_path: format!("jira/{}", ticket.key),
        language: "requirement".to_string(),
        source: DiscoverySource::TicketSystem,
        line_range: None,
    }));

    // Link requirement to mentioned files
    for file_path in &ticket.linked_files {
        events.push(ContextEvent::new(EventPayload::DependencyLinked {
            source_entity: EntityId::local(file_path, &format!("module:{file_path}")),
            target_entity: entity_id.clone(),
            relation_type: RelationType::Implements,
            confidence: 0.70,
            source: DiscoverySource::TicketSystem,
            description: format!("File {file_path} linked to ticket {}", ticket.key),
        }));
    }

    events
}

/// Parse multiple tickets from JSON.
pub fn ingest_tickets_json(json: &str) -> Result<Vec<ContextEvent>, serde_json::Error> {
    let tickets: Vec<JiraTicket> = serde_json::from_str(json)?;
    let mut events = Vec::new();
    for ticket in &tickets {
        events.extend(ingest_ticket(ticket));
    }
    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ingest_ticket() {
        let ticket = JiraTicket {
            key: "AUTH-42".into(),
            summary: "Implement OAuth2 authentication".into(),
            description: "Users should be able to log in via OAuth2 providers".into(),
            acceptance_criteria: vec![
                "Users can log in with Google".into(),
                "Users can log in with GitHub".into(),
                "Session persists for 24 hours".into(),
            ],
            linked_files: vec!["src/auth/service.ts".into()],
            status: "In Progress".into(),
        };

        let events = ingest_ticket(&ticket);
        assert!(events.len() >= 2, "Should create requirement + feature entities");

        // Check that dependency was created
        let deps: Vec<_> = events.iter().filter(|e| matches!(&e.payload, EventPayload::DependencyLinked { .. })).collect();
        assert_eq!(deps.len(), 1, "Should link file to requirement");
    }
}
