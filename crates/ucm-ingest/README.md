# ucm-ingest

[![Crates.io](https://img.shields.io/crates/v/ucm-ingest.svg)](https://crates.io/crates/ucm-ingest)
[![Docs.rs](https://docs.rs/ucm-ingest/badge.svg)](https://docs.rs/ucm-ingest)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)

> Ingestion adapters that turn code, diffs, tickets, and logs into UCM events.

`ucm-ingest` is the front door of the **Unified Context Management (UCM)** engine.
Each adapter normalizes a different real-world source into the same typed
`UcmEntity` / `UcmEdge` vocabulary (from [`ucm-graph-core`](../ucm-core)) so that
everything downstream reasons over one unified graph.

## What it does

- **Polyglot code parser** — extracts functions, modules, and imports from
  **TypeScript, JavaScript, Rust, and Python**. Critically, it emits a `Module`
  entity *before* the import edges that reference it, so intra-project dependencies
  are never silently dropped.
- **Diff parser** — classifies a unified diff into signature changes, body changes,
  added/removed entities, and import changes — the trigger for impact analysis.
- **Source adapters** — convert external systems into requirement/feature/endpoint
  entities:
  - **Jira** (`jira_adapter`) and **Linear** (`linear_adapter`) → `Requirement` + `Feature`
  - **API logs** (`api_log_adapter`) → `ApiEndpoint` entities with traffic-weighted
    confidence and high-error-rate flags
  - **Git history** (`git_history_adapter`) → `CoChanged` edges weighted by co-change frequency

## Quick example

```rust
use ucm_ingest::code_parser::parse_source_code;

// Parse a source file into a Vec<UcmEvent> (entity-discovered + edge events).
let events = parse_source_code(
    "src/auth.ts",
    "export function validateToken() {}\nimport { db } from './db';",
    "typescript",
);

println!("parsed {} events from one file", events.len());
```

## Where it fits

```
sources ──► ucm-ingest ──► Vec<UcmEvent> ──► ucm-events ──► UcmGraph ──► ucm-reason
(code,       (parsers &                      (append-only    (materialized
 diffs,       adapters)                        log)            graph)
 Jira/Linear,
 API logs,
 git history)
```

## Research foundations

- **Stable identity across languages** — Sourcegraph [SCIP](https://github.com/sourcegraph/scip).
- **Historical coupling** — co-change analysis, à la *Mining Version Histories* (Zimmermann et al.).
- **Traffic-driven discovery** — observability-as-input, inspired by production tracing systems.

## License

MIT — see [LICENSE](../../LICENSE).
