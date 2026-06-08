# ucm-events

[![Crates.io](https://img.shields.io/crates/v/ucm-events.svg)](https://crates.io/crates/ucm-events)
[![Docs.rs](https://docs.rs/ucm-events/badge.svg)](https://docs.rs/ucm-events)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)

> Append-only event store and graph projection for the **Unified Context Management (UCM)** engine.

`ucm-events` is the source of truth for every context mutation. Following the
[Datomic](https://docs.datomic.com/) "database as a value" model, facts are only
ever **appended** — never updated in place. The graph is always rebuildable by
replaying the event log, which makes the whole system auditable and time-travelable.

## What it does

- **`EventStore`** — an append-only log with id and stream indexing, checkpointing
  (`events_since_checkpoint` / `advance_checkpoint`), time-bounded `replay`, and
  `causation_chain` traversal so you can reconstruct *why* a fact exists.
- **`GraphProjection`** — deterministic, idempotent fold from `&[UcmEvent]` to a
  `UcmGraph`. Replay the whole log (`replay_all`) or apply incrementally
  (`apply_event` / `apply_batch`).

## Quick example

```rust
use ucm_events::{EventStore, GraphProjection};

// `events` is a Vec<UcmEvent> produced by ucm-ingest adapters.
let mut store = EventStore::new();
store.append_batch(events.clone());

// Materialize the current graph by folding the entire log.
let graph = GraphProjection::replay_all(&events);

println!("events: {}, entities: {}", store.len(), graph.all_entities().len());
```

## Where it fits

`ucm-ingest` adapters emit `UcmEvent`s → `ucm-events` persists and projects them
into a `UcmGraph` (defined in [`ucm-graph-core`](../ucm-core)) → `ucm-reason`
analyzes the materialized graph. Because state is fully replayable, `ucm-observe`
can re-run any past analysis and verify it is deterministic.

## Research foundations

- **Event sourcing** — Martin Fowler, [Event Sourcing](https://martinfowler.com/eaaDev/EventSourcing.html).
- **Database as a value** — [Datomic](https://docs.datomic.com/) immutable, append-only architecture.

> The in-memory store has the same API surface as a durable backend would; for
> production, swap the storage for RocksDB with a column family per stream.

## License

MIT — see [LICENSE](../../LICENSE).
