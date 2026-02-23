# UCM: Autonomous Impact Analysis Engine

## Executive Summary
This project implements an autonomous reasoning engine for software quality assurance. It moves beyond simple static analysis (grep/find references) by implementing a **Bayesian inference model** on top of a polyglot code graph. The system ingests code, builds a dependency graph, and uses reverse graph traversal to probabilistically determine the impact of code changes, generating high-confidence test plans automatically.

## Core Problem & Solution
**The Challenge**: In large microservice architectures, knowing *exactly* what to test after a code change is difficult. Developers often over-test (slow CI) or under-test (production bugs).
**The Solution**: A graph-based reasoning engine that traces dependencies across file and language boundaries (simulated), assigning confidence scores to potential impacts based on edge types (direct call vs. transitive dependency).

## Technical Architecture

### 1. Robust Rust Backend (`ucm-api`, `ucm-reason`)
The core logic is implemented in Rust for performance and type safety.
- **Graph Primitives**: Uses `petgraph` to model Entities (Functions, Classes, APIs) and Edges (Calls, Imports, Inherits).
- **Event Sourcing**: System state is rebuilt from a canonical log of events (`EntityDiscovered`, `EdgeDetected`), ensuring auditability.
- **Axum API**: High-performance async HTTP API serving the frontend.
- **Hexagonal Architecture**: Core logic (`ucm-reason`) is isolated from infrastructure (`ucm-ingest`, `ucm-api`).

### 2. Probabilistic Reasoning Engine
This is the key differentiator. Instead of boolean (yes/no) analysis, it uses **Bayesian Belief Networks**:
- **Confidence Decay**: Impact confidence decays as it traverses the graph.
  - *Direct Call*: 1.0 confidence
  - *Transitive Call (2 hops)*: $1.0 \times 0.9 = 0.9$
  - *Transitive Call (3 hops)*: $0.9 \times 0.9 = 0.81$
- **Explanation Chains**: Every conclusion is backed by a trace. The system explains *why* it flagged an entity:
  > "Service B is impacted (81% confidence) because it imports Utility C, which calls the modified Function A."

### 3. Modern Interactive Dashboard
Built with **React**, **Vite**, **Tailwind CSS v4**, and **React Flow**.
- **Visual Impact Topology**: Interactive node-graph visualization of the system architecture.
- **Impact Simulator**: Allows users to "hypothetically" change any entity and see the ripple effects in real-time.
- **Test Intent Generation**: Automatically suggests "Must Test" scenarios based on the highest-risk impact paths.

## Key Algorithms ("The Research")

### Reverse Impact BFS
To determine what breaks when `Entity A` changes, we perform a **Reverse Breadth-First Search** on the dependency graph.
$$ Impact(E) = P(Change(A) \rightarrow E) $$
We traverse edges $E \rightarrow A$ (Reverse dependencies) to find all upstream consumers.

### Ambiguity Detection
The system identifies when static analysis is insufficient.
- *Dynamic Dispatch detection*: If an interface is changed, all implementations are flagged as "Ambiguous Impact".
- *Loose Coupling*: REST API calls identified by string matching are flagged with lower confidence than direct function calls.

## Deployment
- **Dockerized**: Multi-stage build (Rust builder image + Debian runtime) for minimal footprint (< 50MB).
- **Railway/Vercel Ready**: Configured via `railway.toml` and environment variables for seamless cloud deployment.

## Future Roadmap
- **LSIF/SCIP Ingestion**: Replace the simulated graph with real-time indexing of TypeScript/Rust codebases using the SCIP protocol.
- **LLM Integration**: Use local LLMs to generate the standard English description of *why* a change is risky, enhancing the structured `ExplanationChain`.
