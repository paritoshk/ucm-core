# ucm-reason

[![Crates.io](https://img.shields.io/crates/v/ucm-reason.svg)](https://crates.io/crates/ucm-reason)
[![Docs.rs](https://docs.rs/ucm-reason/badge.svg)](https://docs.rs/ucm-reason)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)

> The reasoning brain of **Unified Context Management (UCM)** — impact analysis, test intent, and explanation chains.

`ucm-reason` answers the questions that make a knowledge graph actionable:

1. **What changed?**
2. **What is impacted?** — reverse BFS over the dependency graph, decaying confidence at each hop
3. **What is *not* impacted?** — and *why* (equally important for scoping)
4. **What should be tested?** — test intent grouped into confidence tiers
5. **What is uncertain?** — ambiguity detection (source conflicts, stale data, requirement drift)
6. **Why?** — every output carries an `ExplanationChain` of `ReasoningStep`s (evidence → inference → confidence)

## What it does

- **`impact`** — `analyze_impact()` runs a reverse breadth-first search from the
  changed entities, attenuating confidence by graph distance, and classifies results
  into direct/indirect impacts plus an explicit *not impacted* set.
- **`intent`** — `generate_test_intent()` turns an impact report into must-test /
  should-test tiers, key risks, coverage gaps, and "decided not to test" rationale.
- **`explanation`** — human-readable derivation chains (`to_narrative()`).
- **`ambiguity`** — flags low-confidence edges, stale data, and source conflicts.

## Quick example

```rust
use ucm_graph_core::{UcmGraph, EntityId};
use ucm_reason::{analyze_impact, generate_test_intent};

fn report(graph: &UcmGraph, changed: &[EntityId]) {
    // analyze_impact(graph, changed_entities, min_confidence, max_depth)
    let impact = analyze_impact(graph, changed, 0.3, 10);
    let intent = generate_test_intent(&impact);

    println!("direct impacts: {}", impact.direct_impacts.len());
    println!("must-test scenarios: {}", intent.high_confidence.len());
}
```

## Why these models

| Model | Problem it solves | Without it |
| --- | --- | --- |
| **Noisy-OR** | Multiple weak signals → one score | Naive multiply (0.8 × 0.7 = 0.56) is over-pessimistic; Noisy-OR gives ≈ 0.94 |
| **Temporal decay** | Facts go stale at different rates | A two-year-old edge weighs the same as yesterday's |
| **Chain confidence** | Impact attenuates with distance | A 10-hop transitive dep looks as risky as a direct import |

## Research foundations

- **Change awareness** — Meta [Predictive Test Selection](https://research.facebook.com/publications/predictive-test-selection/) (ICSE-SEIP 2019): impact attenuates with dependency-graph distance; 33% of tests caught 99.9% of failures.
- **Scale of impact** — Google [TAP](https://research.google/pubs/pub45861/): beyond ~10 hops, testing approaches a 99:1 noise-to-signal ratio.

## License

MIT — see [LICENSE](../../LICENSE).
