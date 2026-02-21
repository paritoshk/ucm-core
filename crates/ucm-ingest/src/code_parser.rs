//! Code parser — extracts entities and relationships from source code.
//!
//! In production, this would use tree-sitter (56+ languages, incremental parsing).
//! This mock parser demonstrates the same API and event flow by parsing
//! TypeScript/JavaScript-like source using regex patterns.
//!
//! What a real tree-sitter integration would add:
//! - Sub-millisecond incremental re-parsing after edits
//! - Error recovery for partial parses
//! - S-expression query system for structured extraction
//! - 56+ language grammars
//!
//! Reference: tree-sitter https://tree-sitter.github.io/tree-sitter/

use ucm_core::entity::*;
use ucm_core::edge::*;
use ucm_core::event::*;

/// Parse source code and extract entities + relationships as events.
///
/// This mock parser handles TypeScript-like patterns:
/// - `function name(` or `async function name(`
/// - `export function`, `export default function`
/// - `class Name {`
/// - `import { ... } from '...'`
/// - `app.get('/route'`, `router.post('/route'`
pub fn parse_source_code(
    file_path: &str,
    source: &str,
    language: &str,
) -> Vec<UcmEvent> {
    let mut events = Vec::new();

    // Extract functions
    for (name, is_async, line_num, line_end) in extract_functions(source) {
        events.push(UcmEvent::new(EventPayload::EntityDiscovered {
            entity_id: EntityId::local(file_path, &name),
            kind: EntityKind::Function {
                is_async,
                parameter_count: 0, // simplified
                return_type: None,
            },
            name: name.clone(),
            file_path: file_path.to_string(),
            language: language.to_string(),
            source: DiscoverySource::StaticAnalysis,
            line_range: Some((line_num, line_end)),
        }));
    }

    // Extract classes
    for (name, line_num) in extract_classes(source) {
        events.push(UcmEvent::new(EventPayload::EntityDiscovered {
            entity_id: EntityId::local(file_path, &name),
            kind: EntityKind::DataModel {
                fields: Vec::new(),
            },
            name: name.clone(),
            file_path: file_path.to_string(),
            language: language.to_string(),
            source: DiscoverySource::StaticAnalysis,
            line_range: Some((line_num, line_num + 10)),
        }));
    }

    // Extract API routes
    for (method, route, handler, line_num) in extract_routes(source) {
        events.push(UcmEvent::new(EventPayload::EntityDiscovered {
            entity_id: EntityId::local(file_path, &format!("{method}:{route}")),
            kind: EntityKind::ApiEndpoint {
                method: method.clone(),
                route: route.clone(),
                handler: handler.clone(),
            },
            name: format!("{method} {route}"),
            file_path: file_path.to_string(),
            language: language.to_string(),
            source: DiscoverySource::StaticAnalysis,
            line_range: Some((line_num, line_num)),
        }));
    }

    // Extract imports → DependencyLinked events
    for (imported_symbols, from_path, line_num) in extract_imports(source) {
        for symbol in &imported_symbols {
            // Link: current file entity → imported entity
            events.push(UcmEvent::new(EventPayload::DependencyLinked {
                source_entity: EntityId::local(file_path, &format!("module:{file_path}")),
                target_entity: EntityId::local(&from_path, symbol),
                relation_type: RelationType::Imports,
                confidence: 0.95,
                source: DiscoverySource::StaticAnalysis,
                description: format!("import {{ {symbol} }} from '{from_path}' at line {line_num}"),
            }));
        }
    }

    events
}

/// Extract function declarations from source.
fn extract_functions(source: &str) -> Vec<(String, bool, usize, usize)> {
    let mut functions = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        // Match patterns like:
        // function name(
        // async function name(
        // export function name(
        // export async function name(
        // const name = async (
        // const name = (

        let is_async = trimmed.contains("async");

        if let Some(name) = extract_function_name(trimmed) {
            // Estimate function end (simple heuristic: next 20 lines or next function)
            let line_end = line_num + 20;
            functions.push((name, is_async, line_num + 1, line_end));
        }
    }

    functions
}

fn extract_function_name(line: &str) -> Option<String> {
    // "function name(" or "async function name("
    let patterns = ["function ", "async function "];
    for pat in &patterns {
        if let Some(pos) = line.find(pat) {
            let after = &line[pos + pat.len()..];
            let name: String = after.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }

    // "const name = (" or "const name = async ("
    if line.starts_with("const ") || line.starts_with("export const ") {
        let after_const = if line.starts_with("export const ") {
            &line[13..]
        } else {
            &line[6..]
        };
        if let Some(eq_pos) = after_const.find('=') {
            let name: String = after_const[..eq_pos].trim().chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() && (after_const[eq_pos..].contains('(') || after_const[eq_pos..].contains("=>")) {
                return Some(name);
            }
        }
    }

    None
}

/// Extract class declarations.
fn extract_classes(source: &str) -> Vec<(String, usize)> {
    let mut classes = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        // "class Name {" or "export class Name {"
        if trimmed.contains("class ") && trimmed.contains('{') {
            let after_class = trimmed.split("class ").nth(1).unwrap_or("");
            let name: String = after_class.chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                classes.push((name, line_num + 1));
            }
        }
    }

    classes
}

/// Extract API route definitions.
fn extract_routes(source: &str) -> Vec<(String, String, String, usize)> {
    let mut routes = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        // Match: app.get('/route', handler) or router.post('/route', ...)
        for method in &["get", "post", "put", "delete", "patch"] {
            let patterns = [
                format!("app.{method}("),
                format!("router.{method}("),
            ];
            for pat in &patterns {
                if trimmed.contains(pat.as_str()) {
                    if let Some(route) = extract_route_path(trimmed) {
                        routes.push((
                            method.to_uppercase(),
                            route,
                            format!("handler_line_{}", line_num + 1),
                            line_num + 1,
                        ));
                    }
                }
            }
        }
    }

    routes
}

fn extract_route_path(line: &str) -> Option<String> {
    // Find string between quotes after (
    let after_paren = line.split('(').nth(1)?;
    let quote_char = if after_paren.contains('\'') { '\'' } else { '"' };
    let parts: Vec<&str> = after_paren.split(quote_char).collect();
    if parts.len() >= 2 {
        Some(parts[1].to_string())
    } else {
        None
    }
}

/// Extract import statements.
fn extract_imports(source: &str) -> Vec<(Vec<String>, String, usize)> {
    let mut imports = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        // import { X, Y } from './path'
        if trimmed.starts_with("import ") && trimmed.contains("from ") {
            let symbols = extract_import_symbols(trimmed);
            if let Some(path) = extract_import_path(trimmed) {
                if !symbols.is_empty() {
                    imports.push((symbols, path, line_num + 1));
                }
            }
        }
    }

    imports
}

fn extract_import_symbols(line: &str) -> Vec<String> {
    // import { X, Y, Z } from ...
    if let Some(start) = line.find('{') {
        if let Some(end) = line.find('}') {
            let symbols_str = &line[start + 1..end];
            return symbols_str.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }

    // import X from ...
    let after_import = line.strip_prefix("import ").unwrap_or("");
    let default_name: String = after_import.chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if !default_name.is_empty() && default_name != "type" {
        return vec![default_name];
    }

    Vec::new()
}

fn extract_import_path(line: &str) -> Option<String> {
    let after_from = line.split("from ").nth(1)?;
    let quote_char = if after_from.contains('\'') { '\'' } else { '"' };
    let parts: Vec<&str> = after_from.split(quote_char).collect();
    if parts.len() >= 2 {
        // Resolve relative path
        let path = parts[1].to_string();
        let resolved = if path.starts_with("./") || path.starts_with("../") {
            // Strip leading ./ and add .ts extension if missing
            let cleaned = path.trim_start_matches("./");
            if cleaned.contains('.') {
                cleaned.to_string()
            } else {
                format!("{cleaned}.ts")
            }
        } else {
            path
        };
        Some(resolved)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_typescript_functions() {
        let source = r#"
import { DatabaseClient } from './db/client';

export async function validateToken(token: string): Promise<boolean> {
    const decoded = jwt.verify(token);
    return decoded.valid;
}

function helperFunction() {
    return true;
}

const arrowFn = (x: number) => x * 2;
"#;

        let events = parse_source_code("src/auth/service.ts", source, "typescript");

        // Should find functions
        let entity_events: Vec<_> = events.iter().filter(|e| matches!(&e.payload, EventPayload::EntityDiscovered { .. })).collect();
        assert!(entity_events.len() >= 2, "Should find at least 2 functions, found {}", entity_events.len());

        // Should find import dependency
        let dep_events: Vec<_> = events.iter().filter(|e| matches!(&e.payload, EventPayload::DependencyLinked { .. })).collect();
        assert!(!dep_events.is_empty(), "Should find import dependencies");
    }

    #[test]
    fn test_parse_api_routes() {
        let source = r#"
app.get('/api/v1/users', getUsers);
app.post('/api/v1/auth/login', handleLogin);
router.delete('/api/v1/sessions/:id', deleteSession);
"#;

        let events = parse_source_code("src/routes.ts", source, "typescript");
        let routes: Vec<_> = events.iter().filter(|e| {
            matches!(&e.payload, EventPayload::EntityDiscovered {
                kind: EntityKind::ApiEndpoint { .. }, ..
            })
        }).collect();

        assert_eq!(routes.len(), 3, "Should find 3 API routes");
    }

    #[test]
    fn test_parse_classes() {
        let source = r#"
export class AuthService {
    constructor(private db: DatabaseClient) {}

    async validate(token: string) {
        return this.db.query(token);
    }
}
"#;

        let events = parse_source_code("src/auth/service.ts", source, "typescript");
        let classes: Vec<_> = events.iter().filter(|e| {
            matches!(&e.payload, EventPayload::EntityDiscovered {
                kind: EntityKind::DataModel { .. }, ..
            })
        }).collect();

        assert_eq!(classes.len(), 1, "Should find 1 class");
    }
}
