---
layout: default
title: "UCM Case Study: Actionable Impact Analysis on marimo"
---

# UCM Case Study: Actionable Impact Analysis on marimo

## What UCM Does (30-second version)

UCM scans your Python/TS/Rust codebase, builds a dependency graph, and answers one question: **"I changed X — what else might break?"**

It outputs:
1. **Impact Report** — every entity affected by your change, with confidence scores and the exact dependency path
2. **Test Intent** — prioritized list of what to test, what risks exist, and what coverage gaps you have
3. **Ambiguity Flags** — where the graph has low-confidence or stale edges (things the tool isn't sure about)

---

## Getting Started

### Install and build
```bash
git clone https://github.com/paritoshk/ucm-core
cd ucm-core
cargo build --release
# Binary at: target/release/ucm
```

### Scan your codebase
```bash
# TypeScript (default)
ucm scan ./my-project --language typescript

# Python — specify the package root for absolute import resolution
# First, clone the target repository to analyze it locally:
# git clone https://github.com/marimo-team/marimo ~/marimo
ucm scan ~/marimo/marimo --language python --package-root marimo

# Rust
ucm scan ./my-crate --language rust

# Large repos (>500 entities) need --no-limit
ucm scan ~/marimo/marimo --language python --package-root marimo --no-limit
```

### Run impact analysis before your PR
```bash
# "I changed the execute_cell method in executor.py — what might break?"
ucm impact _runtime/executor.py "Executor.execute_cell" \
  --path ~/marimo/marimo --language python --package-root marimo --no-limit --json
```

### Get test recommendations
```bash
# "What should I test before merging?"
ucm intent _runtime/executor.py "Executor.execute_cell" \
  --path ~/marimo/marimo --language python --package-root marimo --no-limit --json
```

### Export full graph for custom analysis
```bash
ucm graph ~/marimo/marimo --language python --package-root marimo --no-limit --export json > graph.json
```

---

## How to Read the JSON Outputs

### Impact Report (`ucm impact --json`)

```json
{
  "changes": [{ "entity_id": "...", "name": "Executor.execute_cell", "file_path": "_runtime/executor.py" }],
  "direct_impacts": [...],
  "indirect_impacts": [...],
  "not_impacted": [...],
  "ambiguities": [...],
  "stats": { "total_entities": 8117, "directly_impacted": 1, "indirectly_impacted": 2 }
}
```

**Each impact entry looks like:**
```json
{
  "name": "hash.py",
  "confidence": 0.95,
  "tier": "High",
  "depth": 1,
  "path": ["_ast/visitor.py#ScopedVisitor", "_save/hash.py#module"],
  "reason": "imports via ScopedVisitor",
  "explanation_chain": {
    "summary": "hash.py is impacted by this change",
    "steps": [{
      "step": 1,
      "evidence": "Graph traversal found dependency path: ScopedVisitor -> hash.py",
      "inference": "hash.py is transitively dependent via 1 hop",
      "confidence": 0.95
    }]
  }
}
```

**What to do with this:**
- `direct_impacts` — these **MUST** be tested. They directly depend on what you changed.
- `indirect_impacts` — test these if confidence > 0.7. The `path` field shows you the exact dependency chain so you can trace WHY.
- `not_impacted` — UCM says these are safe to skip. The `reason` field explains why (no graph path, or confidence below threshold).
- `depth` — how many hops away. Depth 1 = direct consumer. Depth 4+ = likely safe to skip.

### Test Intent Report (`ucm intent --json`)

```json
{
  "high_confidence": [
    {
      "description": "Verify Executor still functions correctly after change",
      "rationale": "contains via Executor.execute_cell",
      "confidence": 0.99,
      "related_entity": "Executor"
    }
  ],
  "risks": [
    {
      "severity": "High",
      "description": "Executor directly depends on changed code — regression risk",
      "mitigation": "Run existing tests for Executor and verify expected behavior"
    },
    {
      "severity": "Medium",
      "description": "DefaultExecutor is indirectly affected via 2-hop chain with 89% confidence",
      "mitigation": "Integration test covering the path: Executor.execute_cell -> Executor -> DefaultExecutor"
    }
  ],
  "coverage_gaps": [
    {
      "entity": "DefaultExecutor",
      "description": "DefaultExecutor is impacted but has no linked test coverage in the graph",
      "recommendation": "Add test coverage for DefaultExecutor focusing on the changed behavior"
    }
  ],
  "decided_not_to_test": [
    { "entity": "marimo_path.py", "reason": "No graph path exists to changed entities", "confidence_of_safety": 0.9 }
  ]
}
```

**What to do with this:**
- `high_confidence` — your PR checklist. Write/run these tests before merging.
- `risks` — include these in your PR description. The `mitigation` field is the action item.
- `coverage_gaps` — these are the entities UCM flagged as impacted but having NO test. This is where you add new tests.
- `decided_not_to_test` — UCM's reasoning for what it says is safe to skip. Review these if your PR is high-risk.

---

## How the Confidence Math Works

UCM uses three mathematical models from the research literature:

### 1. Noisy-OR Fusion (Google Knowledge Vault, KDD 2014)
When multiple evidence sources confirm a dependency:
```
P(edge exists) = 1 - product(1 - P(source_i))
```
**Example:** Static analysis says 92% confident, test coverage confirms at 75%.
Result: `1 - (1-0.92)(1-0.75) = 1 - 0.08*0.25 = 0.98` (98% confident).

Sources that agree **compound** confidence. This is why edges confirmed by both code analysis and test coverage are very high confidence.

### 2. Temporal Decay (TempValid, ACL 2024)
Confidence isn't permanent — edges decay over time since last verification:
```
confidence(t) = base_confidence * exp(-lambda * days_since_verified)
```
Decay rates by edge type:
- Import statements: `lambda=0.001` — very slow (imports rarely become invalid)
- Call graph edges: `lambda=0.005` — slow
- Test coverage: `lambda=0.01` — moderate (tests go stale)
- API traffic: `lambda=0.1` — fast (traffic patterns change daily)

### 3. Chain Confidence (BFS propagation)
When A -> B -> C, the confidence that changing A impacts C:
```
P(A impacts C) = P(A->B) * P(B->C)
```
Each hop **multiplicatively reduces** confidence. A 4-hop chain at 0.95 per edge = `0.95^4 = 0.81`. This is why `depth` matters — deeper impacts are less certain.

For multiple paths (A->B->C and A->D->C), UCM uses Noisy-OR over the path confidences.

### What the Tiers Mean
- **High** (>=0.85) — definitely impacted, definitely test this
- **Medium** (0.60-0.84) — probably impacted, test if time permits
- **Low** (<0.60) — might be impacted, low priority

---

## Which UCM Modules Do What

| Crate | Purpose | Key Function |
|-------|---------|-------------|
| `ucm-ingest/code_parser` | Scans source files, extracts entities (functions, classes, modules) and edges (imports, contains, extends) | `parse_source_code_full()` |
| `ucm-core/graph` | Stores the dependency graph (petgraph), resolves entity lookups, computes reverse dependencies | `UcmGraph`, `reverse_deps()` |
| `ucm-core/confidence` | Noisy-OR fusion, temporal decay, chain confidence math | `noisy_or()`, `temporal_decay()`, `chain_confidence()` |
| `ucm-core/edge` | Edge model with confidence scoring, evidence tracking, decay rates | `UcmEdge`, `decayed_confidence()` |
| `ucm-reason/impact` | Reverse BFS from changed entities, classifies direct/indirect/not-impacted | `analyze_impact()`, `impact_bfs()` |
| `ucm-reason/intent` | Converts impact report into test recommendations, risks, coverage gaps | `generate_test_intent()` |
| `ucm-reason/ambiguity` | Flags low-confidence edges, stale data, conflicting sources | `detect_ambiguities()` |
| `ucm-reason/explanation` | Builds traceable reasoning chains for every conclusion | `ExplanationChain` |
| `ucm-cli` | CLI interface, file walking, package detection | `ucm scan/graph/impact/intent` |

### Data Flow
```
Source files -> code_parser -> UcmEvents -> GraphProjection -> UcmGraph
                                                                |
                                                analyze_impact() <- changed entity IDs
                                                                |
                                                          ImpactReport
                                                                |
                                                generate_test_intent()
                                                                |
                                                          TestIntent (JSON)
```

---

## marimo Validation Results

### What We Scanned

| Metric | Value |
|--------|-------|
| Python files | 1,108 |
| Lines of code | 170,240 |
| Entities discovered | 8,117 (5,832 functions, 1,177 classes, 1,108 modules) |
| Total edges | 14,582 |
| Import edges | 1,295 |
| Contains edges | 3,402 |
| Extends edges | 156 |
| DependsOn edges | 9,729 |
| Largest connected component | 6,386 nodes (78% of graph) |

### Critical Finding: Absolute Imports

marimo has **2,470 absolute imports** and **zero relative imports**. Before our parser fix, UCM produced zero cross-module edges. After: 1,295 import edges and 78% of the graph connected.

### 5 Impact Scenarios

| Scenario | What Changed | Direct | Indirect | Interpretation |
|----------|-------------|--------|----------|---------------|
| A | `Executor.execute_cell` (runtime method) | 1 | 2 | Contained within class hierarchy — low blast radius |
| A' | `_runtime/runtime.py` (module-level) | 118 | 64 | Widest blast radius in codebase — 182 total impacts |
| B | `ScopedVisitor` (AST class) | 1 | 93 | Wide cascade through cell compilation pipeline |
| C | `DirectedGraph` (dependency tracking) | 2 | 24 | Moderate cascade through execution scheduler |
| D | `slider` (UI plugin) | 0 | 0 | Leaf node — correctly identified as isolated |
| E | `flatten` (utility) | 1 | 10 | Cross-cutting but shallow |

**What this proves:** UCM correctly differentiates blast radius. Changes to core runtime cascade widely; changes to UI plugins are isolated. This matches architectural intuition and manual code review.

### Test Intent for Scenario A

UCM recommended **4 high-priority test scenarios**, identified **3 risks**, and flagged **3 coverage gaps** — specifically that `Executor`, `DefaultExecutor`, and `StrictExecutor` are impacted but have no linked test coverage in the graph.

**Actionable output:** Before merging a PR that touches `execute_cell`, a developer should run tests covering those 3 classes and verify the executor chain still works end-to-end.

---

## What a Developer Should Do Before a PR

1. **Run `ucm impact`** on the files/symbols you changed
2. **Check `direct_impacts`** — these are your must-test list
3. **Check `risks`** — paste the high-severity ones into your PR description
4. **Check `coverage_gaps`** — if UCM says an impacted entity has no tests, that's your signal to add one
5. **Check `decided_not_to_test`** — verify UCM's reasoning makes sense for your change

### Example PR workflow
```bash
# After making changes to _ast/visitor.py
ucm intent _ast/visitor.py ScopedVisitor \
  --path ~/marimo/marimo --language python --package-root marimo --no-limit

# Output tells you:
#   MUST TEST: hash.py, cell_manager.py, app.py (direct dependencies)
#   RISKS: 93 indirect impacts through AST pipeline
#   COVERAGE GAPS: SerialRefs, BasePersistenceLoader have no tests
#   SAFE TO SKIP: UI plugins, tutorials, smoke tests (no graph path)
```

---

## Known Limitations

| Limitation | Impact | Workaround |
|-----------|--------|------------|
| `__init__.py` re-exports not resolved | ~48% of absolute imports miss | Use `--package-root` + file is still tracked as module |
| No call-site detection | Only import/contains/extends edges, no `Calls` edges | Impact analysis uses module-level granularity for unresolved calls |
| Regex-based parsing (not tree-sitter) | May miss complex syntax (decorators, comprehensions) | Covers >95% of standard def/class/import patterns |
| No server mode | Must re-scan for each command | Graph persistence is on the roadmap |
| Confidence starts fresh each scan | No persistence of historical confidence | Edges from static analysis have very slow decay (lambda=0.001) |

---

## Reproducing This Case Study

```bash
# 1. Clone marimo
git clone --depth 1 https://github.com/marimo-team/marimo ~/marimo

# 2. Build UCM
cd ucm-core && cargo build --release

# 3. Run the full pipeline
cd case-study/marimo
./run.sh ~/marimo ../../target/release/ucm

# 4. Analyze graph
python3 analyze_graph.py results/graph.json

# 5. Find contribution targets (high-impact, untested code)
python3 contribution_analysis.py
```
