# ucm-observe

[![Crates.io](https://img.shields.io/crates/v/ucm-observe.svg)](https://crates.io/crates/ucm-observe)
[![Docs.rs](https://docs.rs/ucm-observe/badge.svg)](https://docs.rs/ucm-observe)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)

> Decision traces and a replay debugger for **Unified Context Management (UCM)**.

`ucm-observe` is what makes UCM's reasoning *auditable*. Every analysis run is
recorded as a structured `DecisionTrace`, and because the underlying state lives
in an append-only event log, any past decision can be **replayed** and checked for
determinism. When someone asks "why did the system recommend testing X?", the
trace contains the full derivation.

## What it does

- **`DecisionTrace`** — captures a single reasoning run: trigger event, graph
  state hash, analyzed entities, ordered `TraceStep`s (operation, input, output,
  confidence), output summary, and duration.
- **`TraceStore`** — in-memory, UUID-indexed storage of traces.
- **`compare_traces` / `ReplayResult`** — re-run a past analysis and surface any
  `Divergence`s field-by-field, so non-determinism is caught immediately.

## Quick example

```rust
use ucm_observe::{TraceStore, compare_traces};

let mut traces = TraceStore::new();

// `original` and `replayed` are DecisionTraces from two runs of the same input.
let result = compare_traces(&original, &replayed);

if !result.is_deterministic {
    for d in &result.divergences {
        println!("diverged at step {}: {} != {}", d.step, d.original_value, d.replayed_value);
    }
}
```

## Where it fits

`ucm-reason` produces analyses; `ucm-observe` wraps them in traces and verifies
that replaying the same event-sourced state (from [`ucm-events`](../ucm-events))
yields the same result. This closes the loop between *what* the engine decided and
*why* — a prerequisite for compliance-grade use cases.

## Research foundations

- **Replay & determinism** — deterministic record/replay debugging (e.g. [rr](https://rr-project.org/)).
- **Auditability** — event-sourced derivation chains ([Datomic](https://docs.datomic.com/)).

## License

MIT — see [LICENSE](../../LICENSE).
