# UCM — Unified Context Model

**Probabilistic impact analysis for code changes.**

UCM builds a Bayesian dependency graph of your codebase and answers:
*"I changed this function — what else might break, and how confident are you?"*

[![CI](https://github.com/paritoshk/ucm-core/actions/workflows/ci.yml/badge.svg)](https://github.com/paritoshk/ucm-core/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Case Study](https://img.shields.io/badge/case%20study-marimo-green)](https://paritoshk.github.io/ucm-core/)

---

## ⚡ TL;DR: What is UCM?

**UCM is not a log collector. It is a Code Impact Analysis Engine.**

UCM acts like a highly intelligent, real-time map of your codebase:
1. **It ingests code and tickets**: It parses your source code (functions, imports, API endpoints) and your Jira/Linear tickets to find entities and relationships. 
2. **It builds a Graph**: It connects all of this together in memory. It knows that "Ticket JIRA-42" requires changes to the `validateToken` function, and that the `processPayment` endpoint imports `validateToken`.
3. **It answers "What if?":** If you ask, *"I am about to change `validateToken`, what else is going to break?"*, UCM runs a Bayesian probability algorithm to highlight every connected piece of code and calculate a "confidence score" that it will be impacted.

### What is the `ucm-api` server for?
The API server exists so that other tools can talk to that graph cleanly:
- Your **React Dashboard** uses the API to visualize the graph and show the impact reports to managers.
- A **GitHub Action (CI/CD)** could ping the API with a git diff to auto-comment on PRs with impact warnings.

Its core job is to sit in the background, hold the codebase graph in memory, listen for new code changes, and answer impact queries.

---

## Install

```bash
cargo install ucm
```

Or build from source:

```bash
git clone https://github.com/paritoshk/ucm-core
cd ucm-core
cargo build --release
```

---

## Quick start

```bash
# Scan a TypeScript project
ucm scan ./src --language typescript

# What breaks if I change validateToken?
ucm impact src/auth/service.ts validateToken

# Get test recommendations for that change
ucm intent src/auth/service.ts validateToken
```

**Example output:**

```
UCM Impact Analysis
====================
  Changed: src/auth/service.ts#validateToken

  DIRECT IMPACTS:
    authMiddleware — 95% confidence
      1. authMiddleware imports validateToken directly (StaticAnalysis)

  INDIRECT IMPACTS:
    processPayment — 76% confidence (2 hops)
      1. processPayment depends on authMiddleware
      2. authMiddleware imports validateToken

  NOT IMPACTED:
    generateReport — 90% safe (No graph path to changed entities)
```

---

## How it works

UCM scans your source files, builds a typed dependency graph, and runs a
**reverse BFS** from the changed entity. Each hop applies confidence decay:

```
confidence(path) = Π edge_weight_i
```

When multiple independent sources confirm the same relationship (static
analysis + API traffic logs), UCM fuses them with **Noisy-OR**:

```
P(edge) = 1 − Π(1 − P(source_i))
```

This means two 80% signals produce 96% confidence — not 64% (naive multiply).
Confidence also decays over time at rates tuned per relationship type (import
statements decay slowly; API traffic patterns decay fast).

---

## CLI reference

```
ucm scan <path> [--language rust|typescript|python]
    Scan source files and print graph statistics.

ucm graph <path> [--export json]
    Show entity list or export full graph as JSON.

ucm impact <file> <symbol> [--min-confidence 0.1] [--max-depth 10] [--json]
    Run reverse BFS from a changed symbol. Print impacted entities with
    confidence scores and explanation chains.

ucm intent <file> <symbol> [--json]
    Same as impact, but formats output as prioritised test scenarios:
    MUST TEST / SHOULD TEST / RISKS / COVERAGE GAPS.
```

---

## REST API

The `ucm-api` binary exposes the same analysis over HTTP (default: `localhost:3001`).

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Liveness check |
| GET | `/graph/entities` | All entities in graph |
| GET | `/graph/edges` | All edges with confidence |
| GET | `/graph/stats` | Entity/edge counts, avg confidence |
| POST | `/ingest/code` | Scan a file path into graph |
| POST | `/impact` | Impact analysis for a change set |
| POST | `/intent` | Test intent for a change set |
| POST | `/linear/connect` | Connect Linear workspace (API key) |
| GET | `/linear/status` | Connection status |
| POST | `/ingest/linear` | Import Linear issues as graph nodes |

```bash
cargo run --bin ucm-api

curl -s http://localhost:3001/health
# {"status":"ok"}

curl -s -X POST http://localhost:3001/impact \
  -H 'Content-Type: application/json' \
  -d '{"changed_entities":[{"file_path":"src/auth/service.ts","symbol":"validateToken"}]}'
```

---

## Dashboard

Interactive UI for exploring the graph and running impact analysis:

```bash
cd dashboard
npm install && npm run dev
# http://localhost:5173
```

Set `VITE_API_URL` to point at a remote `ucm-api` instance.

---

## Architecture

```
ucm-core      — graph types, Bayesian math, SCIP identity  [open-source]
ucm-ingest    — source adapters: code, git, Jira, Linear   [this repo]
ucm-events    — event store + graph projection             [this repo]
ucm-reason    — BFS impact engine, test intent             [this repo]
ucm-observe   — event replay, audit trail                  [this repo]
ucm-api       — Axum REST server                           [this repo]
ucm-cli       — terminal interface                         [this repo]
```

**Event sourcing:** every parser and adapter emits immutable `UcmEvent`s. The
projection replays them to build the graph — any point-in-time state is
reproducible by replaying the event log up to that timestamp.

**SCIP identity:** entities use Sourcegraph SCIP-style strings
(`scip:local/project/0.0.0/src/auth/service.ts#validateToken`), so files can
be re-indexed independently without central ID coordination.

---

## Current limitations

| Item | Status |
|------|--------|
| Parser | Regex-based. Works for extracting functions and import relationships. Not as precise as tree-sitter for complex generics or macros. |
| Graph persistence | In-memory only. Restarting `ucm-api` rebuilds from scratch. |
| Language support | TypeScript, JavaScript, Rust, Python. Other languages return module entities only. |
| Call-site detection | Import edges are detected. Call edges within function bodies are not yet extracted. |

---

## Development

```bash
cargo test --workspace   # run all tests
cargo clippy --workspace # lint
cargo fmt --all          # format
```

---

## License

MIT — see [LICENSE](LICENSE).
