# ucm-cli

[![Crates.io](https://img.shields.io/crates/v/ucm-cli.svg)](https://crates.io/crates/ucm-cli)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)

> Command-line interface for the **Unified Context Management (UCM)** engine. Installs the `ucm` binary.

`ucm-cli` brings the full UCM pipeline to your terminal and CI: scan a codebase,
inspect the dependency graph, and ask "I changed X — what breaks, and what should
I test?" — all offline, with optional JSON output for automation.

## Install

```bash
cargo install ucm-cli      # provides the `ucm` binary
# or, from this workspace:
cargo run -p ucm-cli -- <subcommand>
```

## Subcommands

| Command | What it does |
| --- | --- |
| `ucm scan <path>` | Parse source files and build the dependency graph |
| `ucm graph <path>` | Show graph statistics (optionally `--export`) |
| `ucm impact <file> <symbol>` | Reverse-BFS impact analysis for a changed symbol |
| `ucm intent <file> <symbol>` | Test-intent recommendations from the impact report |

All commands accept `--language` (`typescript` · `javascript` · `rust` · `python`),
`--package-root` (Python absolute-import resolution), and `--no-limit` (research
mode; raises the community 500-entity cap to 50,000).

## Examples

```bash
# Build and summarize a graph for a Rust project
ucm graph ./my-service --language rust

# What is impacted when validateToken changes? (JSON for CI)
ucm impact src/auth.ts validateToken --language typescript --json

# Turn that into a prioritized test plan
ucm intent src/auth.ts validateToken --min-confidence 0.3 --max-depth 10

# Scan a large Python codebase in research mode
ucm scan ./marimo --language python --package-root marimo --no-limit
```

## Where it fits

The CLI is a thin shell over the workspace libraries: it uses
[`ucm-ingest`](../ucm-ingest) to parse code, [`ucm-graph-core`](../ucm-core) +
[`ucm-events`](../ucm-events) to build the graph, and [`ucm-reason`](../ucm-reason)
to produce impact and intent reports. The `--json` output is designed to be wired
into pull-request checks so every change ships with a blast-radius report.

## License

MIT — see [LICENSE](../../LICENSE).
