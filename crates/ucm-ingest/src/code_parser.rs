//! Code parser — extracts entities and relationships from source code.
//!
//! Extracts functions, classes, structs, and import relationships using
//! language-specific pattern matching. Produces `UcmEvent` streams that
//! the graph projection applies to build the dependency graph.
//!
//! Supported languages: TypeScript, JavaScript, Rust, Python.
//!
//! **Edges produced:** Each file gets a `Module` entity. Functions/structs
//! emit `DependsOn` edges to their module. Import statements emit `Imports`
//! edges from the importing module to the imported symbol. This gives the
//! BFS traversal a complete path: `callerFn → callerModule → importedSymbol`.
//!
//! Production upgrade path: replace `extract_functions_*` with tree-sitter
//! grammars for sub-millisecond incremental re-parsing and error recovery.
//! The event API surface stays identical — only the extraction backend changes.

use std::path::Path;
use ucm_core::edge::*;
use ucm_core::entity::*;
use ucm_core::event::*;

/// Parse source code and emit entity + dependency events.
///
/// # Arguments
/// - `file_path` — path relative to project root (used as entity ID component)
/// - `source`    — raw source text
/// - `language`  — "typescript", "javascript", "rust", or "python"
///
/// # Returns
/// Stream of `UcmEvent`s ready for `GraphProjection::apply_event`.
pub fn parse_source_code(file_path: &str, source: &str, language: &str) -> Vec<UcmEvent> {
    let mut events = Vec::new();

    // 1. Emit a module entity for this file so import edges have a valid source.
    let module_id = EntityId::local(file_path, "module");
    events.push(UcmEvent::new(EventPayload::EntityDiscovered {
        entity_id: module_id.clone(),
        kind: EntityKind::Module {
            language: language.to_string(),
            exports: vec![],
        },
        name: file_name_of(file_path),
        file_path: file_path.to_string(),
        language: language.to_string(),
        source: DiscoverySource::StaticAnalysis,
        line_range: None,
    }));

    // 2. Extract function/struct entities and wire them to the module.
    let functions = match language {
        "rust" | "rs" => extract_functions_rust(source),
        "python" | "py" => extract_functions_python(source),
        _ => extract_functions_ts(source),
    };

    for (name, is_async, line_start, line_end) in functions {
        let fn_id = EntityId::local(file_path, &name);
        events.push(UcmEvent::new(EventPayload::EntityDiscovered {
            entity_id: fn_id.clone(),
            kind: EntityKind::Function {
                is_async,
                parameter_count: 0,
                return_type: None,
            },
            name: name.clone(),
            file_path: file_path.to_string(),
            language: language.to_string(),
            source: DiscoverySource::StaticAnalysis,
            line_range: Some((line_start, line_end)),
        }));
        // fn → module: "this function lives in this module"
        events.push(UcmEvent::new(EventPayload::DependencyLinked {
            source_entity: fn_id,
            target_entity: module_id.clone(),
            relation_type: RelationType::DependsOn,
            confidence: 0.99,
            source: DiscoverySource::StaticAnalysis,
            description: format!("{name} is defined in {file_path}"),
        }));
    }

    // 3. Extract class / struct entities.
    let structs = match language {
        "rust" | "rs" => extract_structs_rust(source),
        "python" | "py" => extract_classes_python(source),
        _ => extract_classes_ts(source),
    };

    for (name, line_num) in structs {
        let struct_id = EntityId::local(file_path, &name);
        events.push(UcmEvent::new(EventPayload::EntityDiscovered {
            entity_id: struct_id.clone(),
            kind: EntityKind::DataModel { fields: vec![] },
            name: name.clone(),
            file_path: file_path.to_string(),
            language: language.to_string(),
            source: DiscoverySource::StaticAnalysis,
            line_range: Some((line_num, line_num + 5)),
        }));
        events.push(UcmEvent::new(EventPayload::DependencyLinked {
            source_entity: struct_id,
            target_entity: module_id.clone(),
            relation_type: RelationType::DependsOn,
            confidence: 0.99,
            source: DiscoverySource::StaticAnalysis,
            description: format!("{name} is defined in {file_path}"),
        }));
    }

    // 4. Extract API routes (TypeScript/JS only).
    if matches!(language, "typescript" | "javascript" | "ts" | "js") {
        for (method, route, _handler, line_num) in extract_routes_ts(source) {
            let route_id = EntityId::local(file_path, &format!("{method}:{route}"));
            events.push(UcmEvent::new(EventPayload::EntityDiscovered {
                entity_id: route_id.clone(),
                kind: EntityKind::ApiEndpoint {
                    method: method.clone(),
                    route: route.clone(),
                    handler: String::new(),
                },
                name: format!("{method} {route}"),
                file_path: file_path.to_string(),
                language: language.to_string(),
                source: DiscoverySource::StaticAnalysis,
                line_range: Some((line_num, line_num)),
            }));
            events.push(UcmEvent::new(EventPayload::DependencyLinked {
                source_entity: route_id,
                target_entity: module_id.clone(),
                relation_type: RelationType::DependsOn,
                confidence: 0.99,
                source: DiscoverySource::StaticAnalysis,
                description: format!("{method} {route} is defined in {file_path}"),
            }));
        }
    }

    // 5. Extract imports → module:file imports symbol.
    //    module_id → imported symbol entity.
    //    When the imported symbol changes, BFS propagates to this module,
    //    then to all functions/structs that DependsOn this module.
    let imports = match language {
        "rust" | "rs" => extract_imports_rust(source, file_path),
        "python" | "py" => extract_imports_python(source, file_path),
        _ => extract_imports_ts(source, file_path),
    };

    for (symbols, from_path, line_num) in imports {
        for symbol in &symbols {
            events.push(UcmEvent::new(EventPayload::DependencyLinked {
                source_entity: module_id.clone(),
                target_entity: EntityId::local(&from_path, symbol),
                relation_type: RelationType::Imports,
                confidence: 0.95,
                source: DiscoverySource::StaticAnalysis,
                description: format!("import {symbol} from '{from_path}' at line {line_num}"),
            }));
        }
    }

    events
}

// ── TypeScript / JavaScript ───────────────────────────────────────────────────

fn extract_functions_ts(source: &str) -> Vec<(String, bool, usize, usize)> {
    let mut out = Vec::new();
    for (i, line) in source.lines().enumerate() {
        let t = line.trim();
        let is_async = t.contains("async");
        if let Some(name) = ts_function_name(t) {
            out.push((name, is_async, i + 1, i + 20));
        }
    }
    out
}

fn ts_function_name(line: &str) -> Option<String> {
    for pat in &["function ", "async function "] {
        if let Some(pos) = line.find(pat) {
            let after = &line[pos + pat.len()..];
            let name: String = after
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }
    // const name = ( or const name = async (
    if line.starts_with("const ") || line.starts_with("export const ") {
        let rest = line
            .strip_prefix("export const ")
            .unwrap_or_else(|| line.strip_prefix("const ").unwrap_or(line));
        if let Some(eq) = rest.find('=') {
            let name: String = rest[..eq]
                .trim()
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            let after_eq = &rest[eq..];
            if !name.is_empty() && (after_eq.contains('(') || after_eq.contains("=>")) {
                return Some(name);
            }
        }
    }
    None
}

fn extract_classes_ts(source: &str) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    for (i, line) in source.lines().enumerate() {
        let t = line.trim();
        if t.contains("class ") && t.contains('{') {
            if let Some(after) = t.split("class ").nth(1) {
                let name: String = after
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if !name.is_empty() {
                    out.push((name, i + 1));
                }
            }
        }
    }
    out
}

fn extract_routes_ts(source: &str) -> Vec<(String, String, String, usize)> {
    let mut out = Vec::new();
    for (i, line) in source.lines().enumerate() {
        let t = line.trim();
        for method in &["get", "post", "put", "delete", "patch"] {
            for prefix in &[format!("app.{method}("), format!("router.{method}(")] {
                if t.contains(prefix.as_str()) {
                    if let Some(route) = ts_route_path(t) {
                        out.push((method.to_uppercase(), route, String::new(), i + 1));
                    }
                }
            }
        }
    }
    out
}

fn ts_route_path(line: &str) -> Option<String> {
    let after = line.split('(').nth(1)?;
    let q = if after.contains('\'') { '\'' } else { '"' };
    let parts: Vec<&str> = after.split(q).collect();
    if parts.len() >= 2 {
        Some(parts[1].to_string())
    } else {
        None
    }
}

/// Returns `(symbols, resolved_path, line_number)` for TypeScript imports.
fn extract_imports_ts(source: &str, current_file: &str) -> Vec<(Vec<String>, String, usize)> {
    let mut out = Vec::new();
    let dir = parent_dir(current_file);
    for (i, line) in source.lines().enumerate() {
        let t = line.trim();
        if t.starts_with("import ") && t.contains("from ") {
            let symbols = ts_import_symbols(t);
            if let Some(raw_path) = ts_import_path(t) {
                // Only follow relative imports — node_modules won't be in the graph.
                if raw_path.starts_with("./") || raw_path.starts_with("../") {
                    let resolved = resolve_path(&dir, &raw_path, &["ts", "tsx", "js"]);
                    if !symbols.is_empty() {
                        out.push((symbols, resolved, i + 1));
                    }
                }
            }
        }
    }
    out
}

fn ts_import_symbols(line: &str) -> Vec<String> {
    if let (Some(s), Some(e)) = (line.find('{'), line.find('}')) {
        return line[s + 1..e]
            .split(',')
            .map(|s| {
                s.trim()
                    .split(" as ")
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string()
            })
            .filter(|s| !s.is_empty())
            .collect();
    }
    // default import: import Foo from ...
    let after = line.strip_prefix("import ").unwrap_or("");
    let name: String = after
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if !name.is_empty() && name != "type" {
        vec![name]
    } else {
        vec![]
    }
}

fn ts_import_path(line: &str) -> Option<String> {
    let after = line.split("from ").nth(1)?;
    let q = if after.contains('\'') { '\'' } else { '"' };
    let parts: Vec<&str> = after.split(q).collect();
    if parts.len() >= 2 {
        Some(parts[1].to_string())
    } else {
        None
    }
}

// ── Rust ─────────────────────────────────────────────────────────────────────

fn extract_functions_rust(source: &str) -> Vec<(String, bool, usize, usize)> {
    let mut out = Vec::new();
    for (i, line) in source.lines().enumerate() {
        let t = line.trim();
        // Skip test functions and doc comments
        if t.starts_with("//") || t.starts_with("#[test") {
            continue;
        }
        if let Some(name) = rust_fn_name(t) {
            let is_async = t.contains("async ");
            out.push((name, is_async, i + 1, i + 30));
        }
    }
    out
}

fn rust_fn_name(line: &str) -> Option<String> {
    // Strip visibility and qualifiers
    let stripped = line
        .trim_start_matches("pub(crate) ")
        .trim_start_matches("pub(super) ")
        .trim_start_matches("pub ")
        .trim_start_matches("async ")
        .trim_start_matches("unsafe ")
        .trim_start_matches("extern \"C\" ");
    if let Some(rest) = stripped.strip_prefix("fn ") {
        let name: String = rest
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() {
            return Some(name);
        }
    }
    None
}

fn extract_structs_rust(source: &str) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    for (i, line) in source.lines().enumerate() {
        let t = line.trim();
        let stripped = t
            .trim_start_matches("pub(crate) ")
            .trim_start_matches("pub ");
        if let Some(rest) = stripped.strip_prefix("struct ") {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                out.push((name, i + 1));
            }
        } else if let Some(rest) = stripped.strip_prefix("enum ") {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                out.push((name, i + 1));
            }
        } else if let Some(rest) = stripped.strip_prefix("trait ") {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                out.push((name, i + 1));
            }
        }
    }
    out
}

/// Extract intra-project `use` imports from Rust source.
/// Only follows `use crate::` and `use super::` — skips `std` and external crates.
fn extract_imports_rust(source: &str, _current_file: &str) -> Vec<(Vec<String>, String, usize)> {
    let mut out = Vec::new();
    for (i, line) in source.lines().enumerate() {
        let t = line.trim();
        if !t.starts_with("use ") {
            continue;
        }
        let rest = &t[4..]; // strip "use "
                            // Only care about crate-relative paths
        let (crate_path, rest_after_prefix) = if let Some(r) = rest.strip_prefix("crate::") {
            ("crate", r)
        } else if let Some(r) = rest.strip_prefix("super::") {
            ("super", r)
        } else {
            // Skip std:: and external crate imports
            continue;
        };

        // Normalise: "crate::graph::UcmGraph" → file=crate/graph, symbol=UcmGraph
        // Strip trailing semicolon and braces for glob/multi-import
        let cleaned = rest_after_prefix.trim_end_matches(';');
        let (module_path, symbols) = if cleaned.contains('{') {
            // use crate::foo::{A, B, C}
            let brace_start = cleaned.find('{').unwrap_or(cleaned.len());
            let prefix = cleaned[..brace_start].trim_end_matches(':');
            let inner = cleaned
                .get(brace_start + 1..)
                .and_then(|s| s.split('}').next())
                .unwrap_or("");
            let syms: Vec<String> = inner
                .split(',')
                .map(|s| {
                    s.trim()
                        .split(" as ")
                        .next()
                        .unwrap_or("")
                        .trim()
                        .to_string()
                })
                .filter(|s| !s.is_empty() && s != "*")
                .collect();
            (prefix.to_string(), syms)
        } else {
            // use crate::foo::bar::Baz  →  module=crate/foo/bar, symbol=Baz
            let parts: Vec<&str> = cleaned.split("::").collect();
            if parts.len() < 2 {
                continue;
            }
            let symbol = parts.last().unwrap().to_string();
            if symbol == "*" {
                continue;
            }
            let mod_parts = &parts[..parts.len() - 1];
            (mod_parts.join("::"), vec![symbol])
        };

        if symbols.is_empty() {
            continue;
        }

        // Convert crate::foo::bar → a file path approximation.
        // We don't know the exact file layout, but the symbol name is what matters
        // for matching against EntityId::local(file, symbol) produced by scanning.
        let path_approx = format!("{crate_path}/{}", module_path.replace("::", "/"));
        out.push((symbols, path_approx, i + 1));
    }
    out
}

// ── Python ───────────────────────────────────────────────────────────────────

fn extract_functions_python(source: &str) -> Vec<(String, bool, usize, usize)> {
    let mut out = Vec::new();
    for (i, line) in source.lines().enumerate() {
        let t = line.trim();
        if let Some(rest) = t
            .strip_prefix("async def ")
            .or_else(|| t.strip_prefix("def "))
        {
            let is_async = t.starts_with("async ");
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                out.push((name, is_async, i + 1, i + 20));
            }
        }
    }
    out
}

fn extract_classes_python(source: &str) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    for (i, line) in source.lines().enumerate() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("class ") {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                out.push((name, i + 1));
            }
        }
    }
    out
}

fn extract_imports_python(source: &str, current_file: &str) -> Vec<(Vec<String>, String, usize)> {
    let mut out = Vec::new();
    let dir = parent_dir(current_file);
    for (i, line) in source.lines().enumerate() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("from .") {
            // from .module import Foo, Bar
            if let Some(imp_pos) = rest.find(" import ") {
                let mod_part = &rest[..imp_pos];
                let imp_part = &rest[imp_pos + 8..];
                let symbols: Vec<String> = imp_part
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty() && s != "*")
                    .collect();
                let path = format!("{dir}/{}.py", mod_part.trim_start_matches('.'));
                if !symbols.is_empty() {
                    out.push((symbols, path, i + 1));
                }
            }
        }
    }
    out
}

// ── Path utilities ────────────────────────────────────────────────────────────

fn parent_dir(file_path: &str) -> String {
    Path::new(file_path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

fn file_name_of(file_path: &str) -> String {
    Path::new(file_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| file_path.to_string())
}

/// Resolve a relative import path (e.g., `"./auth/service"`) against the
/// directory of the importing file, appending `.ts` if no extension present.
///
/// Uses `PathBuf::join` + manual component normalization so that `../`
/// traversal works correctly even when `dir` is a single-level path
/// (e.g. `"fraud"` + `"../pipeline/rag"` → `"pipeline/rag.ts"`).
fn resolve_path(dir: &str, raw: &str, _extensions: &[&str]) -> String {
    use std::path::{Component, PathBuf};

    // Build base: treat empty dir as current directory.
    let base = if dir.is_empty() {
        PathBuf::from(".")
    } else {
        PathBuf::from(dir)
    };

    // Join and normalize away `.` and `..` manually.
    // PathBuf::join handles the concatenation; we then walk components to
    // resolve parent-dir traversal without touching the filesystem.
    let joined = base.join(raw);
    let mut parts: Vec<std::ffi::OsString> = Vec::new();
    for comp in joined.components() {
        match comp {
            Component::ParentDir => {
                parts.pop();
            }
            Component::CurDir => {}
            Component::RootDir => {} // drop any accidental leading /
            other => parts.push(other.as_os_str().to_owned()),
        }
    }
    let normalized: PathBuf = parts.iter().collect();
    let s = normalized.to_string_lossy();

    // Append .ts extension if the path has no extension (most TS imports omit it).
    if Path::new(s.as_ref()).extension().is_none() {
        format!("{s}.ts")
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_typescript_emits_module_entity() {
        let source = r#"
import { DatabaseClient } from './db/client';
export async function validateToken(token: string): Promise<boolean> {
    return true;
}
"#;
        let events = parse_source_code("src/auth/service.ts", source, "typescript");

        let entity_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(&e.payload, EventPayload::EntityDiscovered { .. }))
            .collect();
        // module + validateToken
        assert!(
            entity_events.len() >= 2,
            "Expected module + function entities"
        );

        let dep_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(&e.payload, EventPayload::DependencyLinked { .. }))
            .collect();
        // validateToken→module + module→DatabaseClient
        assert!(
            dep_events.len() >= 2,
            "Expected function→module + module→import edges"
        );
    }

    #[test]
    fn test_module_entity_is_discovered_before_import_edges() {
        let source = "import { Foo } from './foo';\nfunction bar() {}";
        let events = parse_source_code("src/main.ts", source, "typescript");

        // Module entity must appear before DependencyLinked so projection has it.
        let first_entity = events
            .iter()
            .position(|e| matches!(&e.payload, EventPayload::EntityDiscovered { .. }));
        let first_dep = events
            .iter()
            .position(|e| matches!(&e.payload, EventPayload::DependencyLinked { .. }));
        assert!(
            first_entity < first_dep,
            "EntityDiscovered must precede DependencyLinked"
        );
    }

    #[test]
    fn test_parse_rust_functions_and_structs() {
        let source = r#"
use crate::graph::UcmGraph;

pub struct GraphProjection;

impl GraphProjection {
    pub fn replay_all(events: &[UcmEvent]) -> UcmGraph {
        UcmGraph::new()
    }

    pub async fn apply_event(graph: &mut UcmGraph, event: &UcmEvent) {}
}
"#;
        let events = parse_source_code("src/projection.rs", source, "rust");

        let entities: Vec<_> = events
            .iter()
            .filter(|e| {
                matches!(
                    &e.payload,
                    EventPayload::EntityDiscovered {
                        kind: EntityKind::Function { .. },
                        ..
                    }
                )
            })
            .collect();
        assert!(
            entities.len() >= 2,
            "Should find replay_all and apply_event"
        );

        let structs: Vec<_> = events
            .iter()
            .filter(|e| {
                matches!(
                    &e.payload,
                    EventPayload::EntityDiscovered {
                        kind: EntityKind::DataModel { .. },
                        ..
                    }
                )
            })
            .collect();
        assert!(!structs.is_empty(), "Should find GraphProjection struct");
    }

    #[test]
    fn test_parse_rust_imports() {
        let source = r#"
use crate::entity::EntityId;
use crate::graph::UcmGraph;
use std::collections::HashMap;
"#;
        let imports = extract_imports_rust(source, "src/main.rs");
        // Only crate:: imports, skip std::
        assert_eq!(imports.len(), 2, "Should find 2 crate imports, skip std");
        assert!(imports
            .iter()
            .any(|(syms, _, _)| syms.contains(&"EntityId".to_string())));
        assert!(imports
            .iter()
            .any(|(syms, _, _)| syms.contains(&"UcmGraph".to_string())));
    }

    #[test]
    fn test_parse_api_routes() {
        let source = r#"
app.get('/api/v1/users', getUsers);
app.post('/api/v1/auth/login', handleLogin);
"#;
        let events = parse_source_code("src/routes.ts", source, "typescript");
        let routes: Vec<_> = events
            .iter()
            .filter(|e| {
                matches!(
                    &e.payload,
                    EventPayload::EntityDiscovered {
                        kind: EntityKind::ApiEndpoint { .. },
                        ..
                    }
                )
            })
            .collect();
        assert_eq!(routes.len(), 2);
    }

    #[test]
    fn test_resolve_path_parent_traversal() {
        // fraud/agent.ts imports from ../pipeline/rag-pipeline
        // dir = "fraud", raw = "../pipeline/rag-pipeline"
        // expected = "pipeline/rag-pipeline.ts"  (NOT "/pipeline/rag-pipeline.ts")
        let result = resolve_path("fraud", "../pipeline/rag-pipeline", &["ts"]);
        assert_eq!(result, "pipeline/rag-pipeline.ts");

        // nested: src/fraud/agent.ts imports from ../pipeline/rag
        // dir = "src/fraud", raw = "../pipeline/rag"
        // expected = "src/pipeline/rag.ts"
        let result2 = resolve_path("src/fraud", "../pipeline/rag", &["ts"]);
        assert_eq!(result2, "src/pipeline/rag.ts");

        // same-dir import: fraud/agent.ts imports ./compliance-checker
        let result3 = resolve_path("fraud", "./compliance-checker", &["ts"]);
        assert_eq!(result3, "fraud/compliance-checker.ts");

        // file at root level: dir = "", raw = "./embedding-service"
        let result4 = resolve_path("", "./embedding-service", &["ts"]);
        assert_eq!(result4, "embedding-service.ts");
    }

    #[test]
    fn test_full_graph_has_edges() {
        // Simulate two files: auth.ts exports validateToken, middleware.ts imports it.
        let auth_src = "export async function validateToken() {}";
        let mid_src =
            "import { validateToken } from './auth';\nexport function authMiddleware() {}";

        use ucm_core::graph::UcmGraph;
        use ucm_events::projection::GraphProjection;
        let mut graph = UcmGraph::new();
        for ev in parse_source_code("src/auth.ts", auth_src, "typescript") {
            GraphProjection::apply_event(&mut graph, &ev);
        }
        for ev in parse_source_code("src/middleware.ts", mid_src, "typescript") {
            GraphProjection::apply_event(&mut graph, &ev);
        }

        let stats = graph.stats();
        assert!(stats.entity_count >= 2, "Should have entities");
        assert!(
            stats.edge_count >= 1,
            "Should have at least one edge — this was the core bug"
        );
    }
}
