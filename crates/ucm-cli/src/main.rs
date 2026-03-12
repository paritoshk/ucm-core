//! UCM CLI — Unified Context Model command-line tool.
//!
//! Provides terminal-based impact analysis, test intent generation,
//! and dependency graph exploration.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use ucm_graph_core::entity::EntityId;
use ucm_graph_core::graph::UcmGraph;
use ucm_ingest::code_parser;
use ucm_reason::ambiguity::enrich_with_ambiguities;
use ucm_reason::impact::analyze_impact;
use ucm_reason::intent::generate_test_intent;

/// UCM community edition entity limit.
/// Full analysis requires UCM Pro for repos exceeding this limit.
const COMMUNITY_ENTITY_LIMIT: usize = 500;

/// Research mode entity limit (effectively unlimited for case studies).
const RESEARCH_ENTITY_LIMIT: usize = 50_000;

#[derive(Parser)]
#[command(
    name = "ucm",
    version,
    about = "Unified Context Model — probabilistic impact analysis"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan source files and build a dependency graph
    Scan {
        /// Directory to scan (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Programming language to parse
        #[arg(short, long, default_value = "typescript")]
        language: String,

        /// Disable entity limit (research mode)
        #[arg(long)]
        no_limit: bool,

        /// Python package root name for absolute import resolution (e.g. "marimo")
        #[arg(long)]
        package_root: Option<String>,
    },

    /// Show graph statistics
    Graph {
        /// Directory to scan
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Export format
        #[arg(long)]
        export: Option<String>,

        /// Programming language
        #[arg(short, long, default_value = "typescript")]
        language: String,

        /// Disable entity limit (research mode)
        #[arg(long)]
        no_limit: bool,

        /// Python package root name for absolute import resolution
        #[arg(long)]
        package_root: Option<String>,
    },

    /// Analyze the impact of changes to a file or entity
    Impact {
        /// The file path containing the changed entity
        file: String,

        /// The symbol name that changed
        symbol: String,

        /// Minimum confidence threshold (0.0-1.0)
        #[arg(long, default_value = "0.1")]
        min_confidence: f64,

        /// Maximum traversal depth
        #[arg(long, default_value = "10")]
        max_depth: usize,

        /// Output as JSON instead of formatted text
        #[arg(long)]
        json: bool,

        /// Directory to scan
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// Programming language
        #[arg(short, long, default_value = "typescript")]
        language: String,

        /// Disable entity limit (research mode)
        #[arg(long)]
        no_limit: bool,

        /// Python package root name for absolute import resolution
        #[arg(long)]
        package_root: Option<String>,
    },

    /// Generate test intent recommendations from impact analysis
    Intent {
        /// The file path containing the changed entity
        file: String,

        /// The symbol name that changed
        symbol: String,

        /// Minimum confidence threshold
        #[arg(long, default_value = "0.1")]
        min_confidence: f64,

        /// Maximum traversal depth
        #[arg(long, default_value = "10")]
        max_depth: usize,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Directory to scan
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// Programming language
        #[arg(short, long, default_value = "typescript")]
        language: String,

        /// Disable entity limit (research mode)
        #[arg(long)]
        no_limit: bool,

        /// Python package root name for absolute import resolution
        #[arg(long)]
        package_root: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan {
            path,
            language,
            no_limit,
            package_root,
        } => cmd_scan(&path, &language, no_limit, &package_root),
        Commands::Graph {
            path,
            export,
            language,
            no_limit,
            package_root,
        } => cmd_graph(&path, export.as_deref(), &language, no_limit, &package_root),
        Commands::Impact {
            file,
            symbol,
            min_confidence,
            max_depth,
            json,
            path,
            language,
            no_limit,
            package_root,
        } => cmd_impact(
            &path,
            &language,
            &file,
            &symbol,
            min_confidence,
            max_depth,
            json,
            no_limit,
            &package_root,
        ),
        Commands::Intent {
            file,
            symbol,
            min_confidence,
            max_depth,
            json,
            path,
            language,
            no_limit,
            package_root,
        } => cmd_intent(
            &path,
            &language,
            &file,
            &symbol,
            min_confidence,
            max_depth,
            json,
            no_limit,
            &package_root,
        ),
    }
}

/// Build a graph by scanning source files in the given directory.
fn build_graph(dir: &PathBuf, language: &str, package_root: &Option<String>) -> UcmGraph {
    let mut graph = UcmGraph::new();

    // Walk the directory for source files
    let extensions: Vec<&str> = match language {
        "typescript" | "ts" => vec!["ts", "tsx"],
        "javascript" | "js" => vec!["js", "jsx"],
        "rust" | "rs" => vec!["rs"],
        "python" | "py" => vec!["py"],
        _ => vec!["ts", "js", "rs", "py"],
    };

    // Build crate map for Rust cross-crate import resolution.
    // Scans for Cargo.toml files and maps crate name → src/ directory path.
    let crate_map = if matches!(language, "rust" | "rs") {
        build_rust_crate_map(dir)
    } else {
        code_parser::RustCrateMap::new()
    };

    // Auto-detect Python package root if not specified.
    let py_pkg_root: code_parser::PythonPackageRoot = if matches!(language, "python" | "py") {
        package_root.clone().or_else(|| detect_python_package_root(dir))
    } else {
        None
    };

    let walker = walk_source_files(dir, &extensions);
    for file_path in &walker {
        let source = match std::fs::read_to_string(file_path) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let relative = file_path
            .strip_prefix(dir)
            .unwrap_or(file_path)
            .to_string_lossy()
            .to_string();

        let events = code_parser::parse_source_code_full(
            &relative,
            &source,
            language,
            &crate_map,
            &py_pkg_root,
        );
        for event in &events {
            ucm_events::projection::GraphProjection::apply_event(&mut graph, event);
        }
    }

    graph
}

/// Auto-detect Python package root by looking for a top-level directory with `__init__.py`.
fn detect_python_package_root(dir: &PathBuf) -> Option<String> {
    // Check pyproject.toml for project name first
    let pyproject = dir.join("pyproject.toml");
    if let Ok(content) = std::fs::read_to_string(&pyproject) {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("name") && trimmed.contains('=') {
                let name = trimmed
                    .split('=')
                    .nth(1)?
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
                if !name.is_empty() {
                    // Package root dir must exist
                    let pkg_dir = dir.join(&name);
                    if pkg_dir.join("__init__.py").exists() {
                        return Some(name);
                    }
                    // Try with hyphens replaced by underscores
                    let underscored = name.replace('-', "_");
                    let pkg_dir2 = dir.join(&underscored);
                    if pkg_dir2.join("__init__.py").exists() {
                        return Some(underscored);
                    }
                }
            }
        }
    }

    // Fallback: look for top-level dirs with __init__.py
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name()?.to_string_lossy().to_string();
                if !name.starts_with('.')
                    && !name.starts_with('_')
                    && name != "tests"
                    && name != "test"
                    && path.join("__init__.py").exists()
                {
                    return Some(name);
                }
            }
        }
    }
    None
}

/// Scan for Cargo.toml files and build a mapping: crate_name → src/ directory path.
/// e.g. "ucm_graph_core" → "ucm-core/src"
fn build_rust_crate_map(dir: &PathBuf) -> code_parser::RustCrateMap {
    let mut map = code_parser::RustCrateMap::new();

    fn scan_for_cargo_tomls(dir: &PathBuf, base: &PathBuf, map: &mut code_parser::RustCrateMap) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    if name.starts_with('.') || name == "target" || name == "node_modules" {
                        continue;
                    }
                    scan_for_cargo_tomls(&path, base, map);
                } else if path.file_name().is_some_and(|n| n == "Cargo.toml") {
                    // Read crate name from Cargo.toml
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Some(name_line) = content.lines().find(|l| l.starts_with("name")) {
                            let crate_name = name_line
                                .split('=')
                                .nth(1)
                                .map(|s| s.trim().trim_matches('"').to_string())
                                .unwrap_or_default();
                            if !crate_name.is_empty() {
                                // Map underscored crate name to src/ path
                                let crate_dir = path.parent().unwrap_or(&path);
                                let src_dir = crate_dir.join("src");
                                if src_dir.exists() {
                                    let relative = src_dir
                                        .strip_prefix(base)
                                        .unwrap_or(&src_dir)
                                        .to_string_lossy()
                                        .to_string();
                                    // Rust uses underscores in import paths
                                    let rust_name = crate_name.replace('-', "_");
                                    map.insert(rust_name, relative);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    scan_for_cargo_tomls(dir, dir, &mut map);
    map
}

fn walk_source_files(dir: &PathBuf, extensions: &[&str]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Skip common non-source directories
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if name.starts_with('.')
                    || name == "node_modules"
                    || name == "target"
                    || name == "dist"
                    || name == "build"
                    || name == "__pycache__"
                {
                    continue;
                }
                files.extend(walk_source_files(&path, extensions));
            } else if let Some(ext) = path.extension() {
                if extensions.iter().any(|e| ext == *e) {
                    files.push(path);
                }
            }
        }
    }
    files
}

fn check_community_limit(graph: &UcmGraph, no_limit: bool) -> bool {
    let limit = if no_limit {
        RESEARCH_ENTITY_LIMIT
    } else {
        COMMUNITY_ENTITY_LIMIT
    };
    let stats = graph.stats();
    if stats.entity_count > limit {
        eprintln!();
        eprintln!(
            "  This repo has {} entities, exceeding the {} limit of {}.",
            stats.entity_count,
            if no_limit { "research mode" } else { "community edition" },
            limit
        );
        if !no_limit {
            eprintln!("  Use --no-limit for research mode, or visit https://ucm.dev/pro for unlimited analysis.");
        }
        eprintln!();
        return false;
    }
    true
}

fn cmd_scan(path: &PathBuf, language: &str, _no_limit: bool, package_root: &Option<String>) {
    println!("Scanning {} for {} files...", path.display(), language);
    let graph = build_graph(path, language, package_root);
    let stats = graph.stats();

    println!();
    println!("  Entities discovered: {}", stats.entity_count);
    println!("  Edges detected:     {}", stats.edge_count);
    println!("  Files tracked:       {}", stats.files_tracked);
    if stats.edge_count > 0 {
        println!(
            "  Avg confidence:      {:.1}%",
            stats.avg_confidence * 100.0
        );
    }
    println!();
    println!("  Graph built successfully. Use `ucm impact` to analyze changes.");
}

fn cmd_graph(path: &PathBuf, export: Option<&str>, language: &str, _no_limit: bool, package_root: &Option<String>) {
    let graph = build_graph(path, language, package_root);
    let stats = graph.stats();

    if let Some("json") = export {
        match graph.to_json() {
            Ok(json) => println!("{json}"),
            Err(e) => eprintln!("Error serializing graph: {e}"),
        }
        return;
    }

    println!("UCM Graph Statistics");
    println!("====================");
    println!("  Entities: {}", stats.entity_count);
    println!("  Edges:    {}", stats.edge_count);
    println!("  Files:    {}", stats.files_tracked);
    if stats.edge_count > 0 {
        println!("  Avg conf: {:.1}%", stats.avg_confidence * 100.0);
    }

    // List entities
    println!();
    println!("Entities:");
    for entity in graph.all_entities() {
        println!("  - {} ({})", entity.name, entity.file_path);
    }
}

#[allow(clippy::too_many_arguments)]
fn cmd_impact(
    path: &PathBuf,
    language: &str,
    file: &str,
    symbol: &str,
    min_confidence: f64,
    max_depth: usize,
    json: bool,
    no_limit: bool,
    package_root: &Option<String>,
) {
    let graph = build_graph(path, language, package_root);

    if !check_community_limit(&graph, no_limit) {
        return;
    }

    let changed = vec![EntityId::local(file, symbol)];
    let mut report = analyze_impact(&graph, &changed, min_confidence, max_depth);
    enrich_with_ambiguities(&mut report, &graph, 0.60);

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_default()
        );
        return;
    }

    // Formatted output
    println!("UCM Impact Analysis");
    println!("====================");
    println!("  Changed: {file}#{symbol}");
    println!();

    if !report.direct_impacts.is_empty() {
        println!("  DIRECT IMPACTS:");
        for impact in &report.direct_impacts {
            println!(
                "    {} — {:.0}% confidence",
                impact.name,
                impact.confidence * 100.0
            );
            for step in &impact.explanation_chain.steps {
                println!("      {}. {}", step.step, step.inference);
            }
        }
        println!();
    }

    if !report.indirect_impacts.is_empty() {
        println!("  INDIRECT IMPACTS:");
        for impact in &report.indirect_impacts {
            println!(
                "    {} — {:.0}% confidence ({} hops)",
                impact.name,
                impact.confidence * 100.0,
                impact.depth
            );
            for step in &impact.explanation_chain.steps {
                println!("      {}. {}", step.step, step.inference);
            }
        }
        println!();
    }

    if !report.not_impacted.is_empty() {
        println!("  NOT IMPACTED:");
        for ni in &report.not_impacted {
            println!(
                "    {} — {:.0}% safe ({})",
                ni.name,
                ni.confidence * 100.0,
                ni.reason
            );
        }
        println!();
    }

    if !report.ambiguities.is_empty() {
        println!("  AMBIGUITIES:");
        for amb in &report.ambiguities {
            println!("    [{}] {}", amb.ambiguity_type, amb.description);
            println!("      Recommendation: {}", amb.recommendation);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn cmd_intent(
    path: &PathBuf,
    language: &str,
    file: &str,
    symbol: &str,
    min_confidence: f64,
    max_depth: usize,
    json: bool,
    no_limit: bool,
    package_root: &Option<String>,
) {
    let graph = build_graph(path, language, package_root);

    if !check_community_limit(&graph, no_limit) {
        return;
    }

    let changed = vec![EntityId::local(file, symbol)];
    let mut report = analyze_impact(&graph, &changed, min_confidence, max_depth);
    enrich_with_ambiguities(&mut report, &graph, 0.60);
    let intent = generate_test_intent(&report);

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&intent).unwrap_or_default()
        );
        return;
    }

    println!("UCM Test Intent");
    println!("================");
    println!(
        "  {} scenarios total ({} high, {} medium, {} low)",
        intent.summary.total_scenarios,
        intent.summary.high_count,
        intent.summary.medium_count,
        intent.summary.low_count,
    );
    println!();

    if !intent.high_confidence.is_empty() {
        println!("  MUST TEST:");
        for s in &intent.high_confidence {
            println!("    [{}%] {}", (s.confidence * 100.0) as u32, s.description);
        }
        println!();
    }

    if !intent.medium_confidence.is_empty() {
        println!("  SHOULD TEST:");
        for s in &intent.medium_confidence {
            println!("    [{}%] {}", (s.confidence * 100.0) as u32, s.description);
        }
        println!();
    }

    if !intent.risks.is_empty() {
        println!("  RISKS:");
        for r in &intent.risks {
            println!(
                "    [{:?}] {} — {}",
                r.severity, r.description, r.mitigation
            );
        }
        println!();
    }

    if !intent.coverage_gaps.is_empty() {
        println!("  COVERAGE GAPS:");
        for g in &intent.coverage_gaps {
            println!("    {}: {}", g.entity, g.description);
        }
    }
}
