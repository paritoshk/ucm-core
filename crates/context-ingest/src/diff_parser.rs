//! Git diff parser — compares before/after source to emit ChangeDetected events.
//!
//! In production, this would use git2 (libgit2 bindings) and tree-sitter
//! to compare two ASTs. This mock parser compares source strings and
//! classifies changes semantically.

use context_core::entity::*;
use context_core::event::*;

/// Compare before/after source code and emit change events.
///
/// Classifies changes semantically:
/// - Signature changes (function parameters, return types)
/// - Body changes (implementation details)
/// - New/removed entities
/// - Import changes
pub fn parse_diff(
    file_path: &str,
    before: &str,
    after: &str,
) -> Vec<ContextEvent> {
    let mut events = Vec::new();

    let before_fns = extract_function_signatures(before);
    let after_fns = extract_function_signatures(after);

    // Detect signature changes
    for (name, sig) in &after_fns {
        if let Some(old_sig) = before_fns.get(name.as_str()) {
            if sig != old_sig {
                events.push(ContextEvent::new(EventPayload::ChangeDetected {
                    file_path: file_path.to_string(),
                    change_type: ChangeType::SignatureChange,
                    affected_entities: vec![EntityId::local(file_path, name)],
                    before_snapshot: Some(old_sig.clone()),
                    after_snapshot: Some(sig.clone()),
                }));
            }
        } else {
            // New function
            events.push(ContextEvent::new(EventPayload::ChangeDetected {
                file_path: file_path.to_string(),
                change_type: ChangeType::EntityAdded,
                affected_entities: vec![EntityId::local(file_path, name)],
                before_snapshot: None,
                after_snapshot: Some(sig.clone()),
            }));
        }
    }

    // Detect removed functions
    for (name, sig) in &before_fns {
        if !after_fns.contains_key(name.as_str()) {
            events.push(ContextEvent::new(EventPayload::ChangeDetected {
                file_path: file_path.to_string(),
                change_type: ChangeType::EntityDeleted,
                affected_entities: vec![EntityId::local(file_path, name)],
                before_snapshot: Some(sig.clone()),
                after_snapshot: None,
            }));
        }
    }

    // Detect import changes
    let before_imports = extract_import_lines(before);
    let after_imports = extract_import_lines(after);
    if before_imports != after_imports {
        events.push(ContextEvent::new(EventPayload::ChangeDetected {
            file_path: file_path.to_string(),
            change_type: ChangeType::ImportChange,
            affected_entities: vec![],
            before_snapshot: Some(before_imports.join("\n")),
            after_snapshot: Some(after_imports.join("\n")),
        }));
    }

    // If no specific changes detected but content differs, emit body change
    if events.is_empty() && before != after {
        events.push(ContextEvent::new(EventPayload::ChangeDetected {
            file_path: file_path.to_string(),
            change_type: ChangeType::BodyChange,
            affected_entities: vec![],
            before_snapshot: None,
            after_snapshot: None,
        }));
    }

    events
}

/// Extract function signatures (name → signature string).
fn extract_function_signatures(source: &str) -> std::collections::HashMap<String, String> {
    let mut sigs = std::collections::HashMap::new();

    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.contains("function ") && trimmed.contains('(') {
            let parts: Vec<&str> = trimmed.split("function ").collect();
            if parts.len() >= 2 {
                let after = parts.last().unwrap();
                let name: String = after.chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if !name.is_empty() {
                    // Extract up to the closing paren
                    let sig = if let Some(end) = trimmed.find('{') {
                        trimmed[..end].trim().to_string()
                    } else {
                        trimmed.to_string()
                    };
                    sigs.insert(name, sig);
                }
            }
        }
    }

    sigs
}

/// Extract import lines for comparison.
fn extract_import_lines(source: &str) -> Vec<String> {
    source.lines()
        .filter(|line| line.trim().starts_with("import "))
        .map(|line| line.trim().to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_signature_change() {
        let before = r#"
function validateToken(token: string): boolean {
    return jwt.verify(token);
}
"#;
        let after = r#"
function validateToken(token: string): Result<Claims, AuthError> {
    return jwt.verify(token);
}
"#;

        let events = parse_diff("src/auth/service.ts", before, after);
        assert!(!events.is_empty());

        let change = &events[0];
        match &change.payload {
            EventPayload::ChangeDetected { change_type, .. } => {
                assert!(matches!(change_type, ChangeType::SignatureChange));
            }
            _ => panic!("Expected ChangeDetected event"),
        }
    }

    #[test]
    fn test_detect_new_function() {
        let before = "function existing() {}";
        let after = "function existing() {}\nfunction newFunction() {}";

        let events = parse_diff("src/main.ts", before, after);
        let added: Vec<_> = events.iter().filter(|e| matches!(
            &e.payload,
            EventPayload::ChangeDetected { change_type: ChangeType::EntityAdded, .. }
        )).collect();

        assert!(!added.is_empty());
    }

    #[test]
    fn test_detect_removed_function() {
        let before = "function toRemove() {}\nfunction toKeep() {}";
        let after = "function toKeep() {}";

        let events = parse_diff("src/main.ts", before, after);
        let removed: Vec<_> = events.iter().filter(|e| matches!(
            &e.payload,
            EventPayload::ChangeDetected { change_type: ChangeType::EntityDeleted, .. }
        )).collect();

        assert!(!removed.is_empty());
    }
}
