# ContextQA: Autonomous Impact Reasoning Engine

> "Don't just count the lines of code changed. Reason about the blast radius."

**ContextQA** is a probabilistic reasoning engine that predicts the impact of code changes on your software architecture. It moves beyond static analysis (grep/find references) by building a **Bayesian Belief Network** of your system, ingesting data from code, Jira, API logs, and git history to generate prioritized **Test Intent**.

---

## 🎯 How to Evaluate This Submission

Since I am not sharing the full GitHub repository, this README and the embedded sample output below **are the primary deliverables**.

**Look for:**
1.  **The "Research"**: See [Probabilistic Reasoning](#-probabilistic-reasoning) for the Noisy-OR and Bayesian fusion math used to calculate confidence.
2.  **The Architecture**: A clean separation between the Event Sourced ingestion layer (`context-ingest`), the Graph Core (`context-core`), and the Reasoning Engine (`context-reason`).
3.  **The Output**: Check the [Live Demo Output](#-live-demo-output-real-api-response) section. This is real JSON form the running engine, showing how it handles ambiguity and impact decay.

---

## 🚀 Quick Start

### 1. Run the Backend (Rust)
The engine is written in Rust for performance and correctness.
```bash
cargo run --bin context-api
```
*Starts HTTP server on `localhost:3001` and seeds a demo graph.*

### 2. Run the Dashboard (React)
```bash
cd dashboard
pnpm install && pnpm dev
```
*Starts interactive UI on `localhost:5173`.*

### 3. Run the Tests
```bash
cargo test --workspace
```
**Status**: ✅ **51 tests passed** across 6 crates (Core, Ingest, Reason, Events, Observe, API).

---

## 🧠 Probabilistic Reasoning

Most impact analysis tools satisfy themselves with boolean reachability: "A calls B, so B is impacted." ContextQA implements a **Bayesian** approach because software relationships are rarely certain.

### 1. Noisy-OR Fusion
When multiple evidence sources (e.g., Static Code Analysis AND API Traffic) suggest the same relationship, we don't just average them. We use the **Noisy-OR** model to calculate the probability that *at least one* cause is active.

$$ P(Edge) = 1 - \prod_{i} (1 - P(Source_i)) $$

*Example:* If Static Analysis detects a call (80% confidence) and API Logs also see traffic (60% confidence):
$$ P = 1 - (1 - 0.80)(1 - 0.60) = 1 - (0.2)(0.4) = 0.92 $$
The combined confidence (92%) is higher than either single source, reflecting independent verification.

### 2. Transitive Confidence Decay
Impact confidence decays as it traverses the graph. 
$$ Confidence(Path) = \prod EdgeWeight_i $$
A generic 3-hop dependency is less risky than a direct call. The engine prunes paths that drop below a configurable `min_confidence` threshold.

---

## 💻 Live Demo Output (Real API Response)

Scenario: We changed the return signature of `validateToken()` in `src/auth/service.ts`.
Here is the raw JSON response from the `/intent` endpoint, showing how the engine reasons about downstream risk.

```json
{
  "summary": {
    "total_scenarios": 8,
    "high_count": 3,
    "medium_count": 4,
    "risk_count": 6
  },
  "high_confidence": [
    {
      "description": "Verify authMiddleware() still functions correctly after change",
      "confidence": 0.95,
      "related_entity": "authMiddleware()",
      "rationale": "imports via src/auth/service.ts#validateToken (🟢)",
      "explanation_chain": {
        "summary": "authMiddleware() is impacted by this change",
        "steps": [
          {
            "step": 1,
            "evidence": "Graph traversal found dependency path: validateToken → authMiddleware",
            "inference": "authMiddleware() is transitively dependent via 1 hops",
            "confidence": 0.95
          }
        ]
      }
    }
  ],
  "risks": [
    {
      "severity": "High",
      "description": "Ambiguity: Low confidence (45%) on relationship: generateReport() → getUserProfile()",
      "mitigation": "Verify the relationship between generateReport() and getUserProfile()"
    },
    {
      "severity": "High",
      "description": "JIRA-AUTH-42: OAuth2 Migration directly depends on changed code",
      "mitigation": "Run existing tests for JIRA-AUTH-42"
    }
  ],
  "low_confidence": [
    {
      "description": "Verify generateReport() end-to-end flow still works",
      "confidence": 0.36,
      "related_entity": "generateReport()",
      "rationale": "calls via src/users/profile.ts#getUserProfile (🔴)"
    }
  ]
}
```

**Observation:**
1.  **Ambiguity Detection**: The engine flagged a 45% confidence link (`generateReport` -> `getUserProfile`) detected via API logs. It treats this as a **Risk** to be verified, not a definite fact.
2.  **Jira Integration**: It correctly identified that a Requirement (`JIRA-AUTH-42`) is impacted by the code change in `validateToken`.

---

## 🏗 Architecture & Design Decisions

### 1. Event Sourcing (The "Time Travel" Debugger)
Instead of mutating the graph directly, `context-ingest` emits immutable events (`EntityDiscovered`, `EdgeDetected`).
*   **Why**: We can replay the stream to any point in time to debug "why did the engine think X was impacted yesterday?".
*   **Tradeoff**: Rebuild time grows linearly with history (mitigated by snapshots).

### 2. SCIP Identity vs. UUIDs
We use [SCIP](https://github.com/sourcegraph/scip) style identifiers:
`scip:local/project/0.0.0/src/auth/service.ts#validateToken`
*   **Why**: Decentralized identity. A parser can generate an ID for a function without checking a central database registry.
*   **Tradeoff**: Long string keys increase memory usage compared to integer IDs.

### 3. Hexagonal Architecture
*   `context-core`: Pure domain logic (Graph, Bayesian math). Zero dependencies on HTTP or DB.
*   `context-api`: The "Driving" adapter (Axum).
*   `context-ingest`: The "Driven" adapters (Parsers, Jira).
*   **Why**: We can swap the backend API for a CLI or a WASM module without changing the core reasoning logic.

---

## 🔮 Future Improvements

While this is a robust POC, a production version would include:
1.  **Tree-sitter Integration**: Currently using mock parsers for the demo. Swapping in `tree-sitter` would allow real-time parsing of Rust/TS/Python.
2.  **Graph Persistence**: Currently in-memory. I would add `sled` or `RocksDB` for embedded persistence.
3.  **LLM Narrative Generation**: Feeding the `ExplanationChain` JSON into a local LLM to generate "Human" summary paragraphs.
