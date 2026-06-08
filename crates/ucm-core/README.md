# ucm-graph-core

[![Crates.io](https://img.shields.io/crates/v/ucm-graph-core.svg)](https://crates.io/crates/ucm-graph-core)
[![Docs.rs](https://docs.rs/ucm-graph-core/badge.svg)](https://docs.rs/ucm-graph-core)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)

> Core graph types, entity model, and confidence math for the **Unified Context Management (UCM)** engine.

`ucm-graph-core` is the foundation crate of the UCM workspace. It defines the
identity scheme, the relationship model, the probabilistic confidence functions,
and the in-memory graph that every other crate builds on. It has no I/O and no
async — it is pure data structures and math, so it stays fast and easy to test.

## What it does

- **Stable entity identity** — `EntityId` uses a [SCIP](https://github.com/sourcegraph/scip)-style
  scheme (`scheme:package:descriptor`) so an entity keeps the same identity across
  re-indexing runs, enabling independent file re-ingestion without rewriting the graph.
- **Typed entities & relationships** — `UcmEntity` / `EntityKind` (Function, ApiEndpoint,
  DataModel, Feature, TestCase, Requirement, Module, …) and `UcmEdge` / `RelationType`
  (Imports, Calls, TestedBy, Implements, DependsOn, CoChanged, …).
- **Calibrated confidence** — every edge carries a confidence score fused from multiple
  `EvidenceSource`s via **Noisy-OR**, with **temporal decay** so stale evidence loses weight.
- **Mutable graph** — `UcmGraph` wraps a petgraph `StableGraph` with an entity index and
  Glean-style file ownership tracking, so a single file can be invalidated and re-added.

## Quick example

```rust
use ucm_graph_core::{
    UcmGraph, UcmEntity, EntityId, EntityKind, UcmEdge, RelationType, DiscoverySource,
};

let mut graph = UcmGraph::new();

let token_id = EntityId::new("npm", "my-app", "1.0.0", "src/auth.ts", "validateToken");
let pay_id = EntityId::new("npm", "my-app", "1.0.0", "src/pay.ts", "processPayment");

graph.upsert_entity(UcmEntity::new(
    token_id.clone(), EntityKind::Function, "validateToken", "src/auth.ts", "typescript",
    DiscoverySource::StaticAnalysis,
));
graph.upsert_entity(UcmEntity::new(
    pay_id.clone(), EntityKind::Function, "processPayment", "src/pay.ts", "typescript",
    DiscoverySource::StaticAnalysis,
));

// Link them with an evidence-backed, confidence-scored edge.
let edge = UcmEdge::new(RelationType::Calls, DiscoverySource::StaticAnalysis, 0.95, "static call");
graph.add_relationship(&pay_id, &token_id, edge).unwrap();

println!("entities: {}", graph.all_entities().len());
```

### Confidence math

```rust
use ucm_graph_core::{noisy_or, temporal_decay};

// Two independent sources confirming the same edge compound, not cancel.
let fused = noisy_or(&[0.8, 0.7]); // ≈ 0.94, not 0.56

// Evidence decays exponentially with time since it was last verified.
// temporal_decay(base_confidence, lambda, days_since_verification)
let decayed = temporal_decay(0.95, 0.01, 30.0);
```

## Where it fits

```
ucm-graph-core  ← you are here (types + math, no I/O)
   ├── ucm-events     event sourcing / projection
   ├── ucm-ingest     parsers & adapters that produce entities/edges
   ├── ucm-reason     impact analysis & test intent over the graph
   ├── ucm-observe    decision traces & replay
   └── ucm-api        HTTP surface
```

## Research foundations

- **Calibrated trust** — Google [Knowledge Vault](https://research.google/pubs/pub45634/) (KDD 2014): multi-source fusion where agreement compounds confidence.
- **Stable identity** — Sourcegraph [SCIP](https://github.com/sourcegraph/scip) code-intelligence protocol.
- **Ownership tracking** — Meta [Glean](https://glean.software/) incremental indexing.

## License

MIT — see [LICENSE](../../LICENSE).
