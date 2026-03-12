# UCM vs marimo Case Study — Task Tracker

## Phase 1: Fix Critical Parser Gap (P0 Blocker)
- [x] 1.1 Add absolute Python import parsing (`from marimo.x.y import Z`, `import marimo.x.y`)
- [x] 1.2 Add `--no-limit` CLI flag for research mode
- [x] 1.3 Add Python class-method association (indentation tracking, `Contains` edges)
- [x] 1.4 Add Python inheritance edges (`class Foo(Bar):` → `Extends` edge)
- [x] 1.5 Add unit tests for all new parser features (9 new tests)
- [x] 1.6 `cargo test --workspace` passes (63 tests, 0 failures)
- [x] 1.7 `cargo clippy --workspace` clean (0 warnings)

## Phase 2: Infrastructure Setup
- [x] 2.1 Clone marimo repo (4,257 files)
- [x] 2.2 Create case study directory structure
- [x] 2.3 Profile marimo codebase (1,108 .py files, 170K LOC)

## Phase 3: Run UCM Against marimo
- [x] 3.1 Initial scan: 8,117 entities, 14,582 edges, 1,108 files
- [x] 3.2 Import analysis: 2,470 absolute, 0 relative; 1,295 resolved (52%)
- [x] 3.3 Export and analyze graph (graph.json + analyze_graph.py)

## Phase 4: Impact Analysis Scenarios (5 Cases)
- [x] 4.A Executor.execute_cell: 1 direct, 2 indirect (contained)
- [x] 4.B ScopedVisitor: 1 direct, 93 indirect (wide cascade)
- [x] 4.C DirectedGraph: 2 direct, 24 indirect (moderate)
- [x] 4.D slider: 0 direct, 0 indirect (isolated leaf - correct!)
- [x] 4.E flatten: 1 direct, 10 indirect (wide but shallow)
- [x] 4.1 Test intent generation for Scenario A: 4 high-priority scenarios

## Phase 5: Contribution Discovery
- [x] 5.1 In-degree analysis identifies high-impact entities
- [x] 5.2 Contribution analysis script (running)
- [x] 5.3 Candidate contributions identified in CASE_STUDY.md

## Phase 6: Write-up & Reproducibility
- [x] 6.1 Case study document (CASE_STUDY.md)
- [x] 6.2 Reproducibility scripts (run.sh, validate.py, analyze_graph.py)
