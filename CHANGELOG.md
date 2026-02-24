# Changelog

All notable changes to UCM are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added
- Linear integration: `POST /linear/connect`, `GET /linear/status`, `POST /ingest/linear`
- Integrations tab in dashboard (API key → Connect → Import Issues)
- `ucm-cli` crate with `scan`, `graph`, `impact`, `intent` subcommands
- GitHub Actions CI: test + clippy + fmt on Ubuntu and macOS
- Rust entity extraction (`fn`, `struct`, `enum`, `trait`) in code parser
- Module entities and function→module edges — parser now produces real edges
- Intra-project `use crate::` import edges for Rust source
- Relative import resolution for TypeScript (`./foo` → `src/foo.ts`)

### Fixed
- **Critical:** parser emitted `DependencyLinked` events with a source entity that
  was never `EntityDiscovered`, causing all import edges to be silently dropped.
  Fixed by emitting a `Module` entity per file before any edge events.
- `upsert_entity` unwrap replaced with a descriptive `expect`

### Changed
- Moved `impact_bfs` and `find_not_impacted` from `ucm-core` to `ucm-reason`
  to establish the open-core IP boundary
- `ucm-core` public graph accessors (`inner()`, `entity_node_index()`) added
  for use by external analysis crates

## [0.1.0] — 2026-02-20

### Added
- Initial workspace: `ucm-core`, `ucm-ingest`, `ucm-events`, `ucm-reason`,
  `ucm-observe`, `ucm-api`
- Bayesian confidence scoring: Noisy-OR fusion, temporal decay, chain propagation
- Reverse BFS impact analysis with per-hop confidence decay
- Test intent generation from impact reports
- Event sourcing with idempotent replay (Datomic-inspired)
- SCIP-style entity identifiers (Sourcegraph protocol)
- Adapters: code parser, git diff, Jira, API log, git history
- Axum REST API: `/health`, `/impact`, `/intent`, `/graph/*`, `/ingest/*`
- React dashboard: Architecture, Demo, Data Flow, Impact Simulator, Integrations tabs
- Rebrand from ContextQA to UCM (Unified Context Model)
