# UCM: Reasoning Through a Change

How the UCM system analyzes a code change, determines impact, and generates test intent -- demonstrated with a real API trace.

---

## 1. Demo Graph

Seven entities connected by five directed edges. The graph captures code-level dependencies, API bindings, and external requirement links.

```
                    +---------------------------+
                    | JIRA-AUTH-42              |
                    | OAuth2 Migration          |
                    | (Requirement)             |
                    +------------+--------------+
                                 |
                          RequiredBy (0.70)
                                 |
                                 v
                    +---------------------------+
          +-------->| validateToken()           |<--- CHANGED
          |         | src/auth/service.ts       |     (return type: bool -> Result<Claims, AuthError>)
          |         | Function (async, 1 param) |
          |         +---------------------------+
          |
    Imports (0.95)
          |
          |         +---------------------------+
          +---------| authMiddleware()          |
                    | src/api/middleware.ts      |
                    | Function (async, 2 params)|
                    +--+-----+------------------+
                       |     |
            Calls (0.85)     Calls (0.80)
                       |     |
                       v     v
  +--------------------+     +----------------------+
  | getUserProfile()   |     | processPayment()     |
  | src/users/profile  |     | src/payments/checkout |
  | Function (async)   |     | Function (async)      |
  +--------------------+     +----------+------------+
                                        ^
                                        |
                              Implements (0.92)
                                        |
                             +----------+-----------+
                             | POST /api/checkout   |
                             | src/routes/checkout   |
                             | ApiEndpoint           |
                             +----------------------+


  (No edges)
  +---------------------------+
  | generateReport()          |
  | src/admin/reports.ts      |
  | Function (sync, 0 params) |
  +---------------------------+
```

### Entity Table

| # | Entity               | File                       | Kind         | Notes              |
|---|----------------------|----------------------------|--------------|--------------------|
| 1 | validateToken()      | src/auth/service.ts        | Function     | async, 1 param     |
| 2 | authMiddleware()     | src/api/middleware.ts       | Function     | async, 2 params    |
| 3 | processPayment()     | src/payments/checkout.ts   | Function     | async, 1 param     |
| 4 | getUserProfile()     | src/users/profile.ts       | Function     | async, 1 param     |
| 5 | generateReport()     | src/admin/reports.ts       | Function     | sync, 0 params     |
| 6 | POST /api/checkout   | src/routes/checkout.ts     | ApiEndpoint  |                    |
| 7 | JIRA-AUTH-42         | jira                       | Requirement  | OAuth2 Migration   |

### Edge Table

| Source              | Target           | Relation    | Confidence |
|---------------------|------------------|-------------|------------|
| authMiddleware()    | validateToken()  | Imports     | 0.95       |
| processPayment()   | authMiddleware() | Calls       | 0.80       |
| getUserProfile()   | authMiddleware() | Calls       | 0.85       |
| POST /api/checkout | processPayment() | Implements  | 0.92       |
| JIRA-AUTH-42       | validateToken()  | RequiredBy  | 0.70       |

---

## 2. Change Scenario

**What changed:** `validateToken()` in `src/auth/service.ts`.

**Nature of the change:** The return type shifts from a plain `boolean` to `Result<Claims, AuthError>`. This is a breaking signature change -- every call site that previously checked a boolean must now unwrap a Result type and handle both the success variant (which now carries a `Claims` payload) and the error variant.

**Why it matters:** This is not a behavioral bugfix or a performance tweak. It is a contract change. Any code that imports or calls `validateToken()` will fail to compile (in a typed language) or fail at runtime (in a dynamic one) unless it is updated to handle the new return shape. The blast radius fans out through every entity that transitively depends on the token validation contract.

---

## 3. Impact Analysis

Output from `POST /impact`, cleaned up for readability.

### Response JSON

```json
{
  "change": {
    "entity": "validateToken()",
    "file": "src/auth/service.ts",
    "description": "Return type changed from boolean to Result<Claims, AuthError>"
  },
  "direct_impacts": [
    {
      "entity": "authMiddleware()",
      "file": "src/api/middleware.ts",
      "confidence": 0.949,
      "hops": 1,
      "reason": "Imports validateToken directly",
      "edge": "Imports (0.95)"
    },
    {
      "entity": "JIRA-AUTH-42: OAuth2 Migration",
      "source": "jira",
      "confidence": 0.694,
      "hops": 1,
      "reason": "Required by validateToken",
      "edge": "RequiredBy (0.70)"
    }
  ],
  "indirect_impacts": [
    {
      "entity": "getUserProfile()",
      "file": "src/users/profile.ts",
      "confidence": 0.802,
      "hops": 2,
      "path": ["validateToken()", "authMiddleware()", "getUserProfile()"],
      "reason": "Calls authMiddleware, which imports validateToken"
    },
    {
      "entity": "processPayment()",
      "file": "src/payments/checkout.ts",
      "confidence": 0.755,
      "hops": 2,
      "path": ["validateToken()", "authMiddleware()", "processPayment()"],
      "reason": "Calls authMiddleware, which imports validateToken"
    },
    {
      "entity": "POST /api/checkout",
      "file": "src/routes/checkout.ts",
      "confidence": 0.689,
      "hops": 3,
      "path": ["validateToken()", "authMiddleware()", "processPayment()", "POST /api/checkout"],
      "reason": "Implements processPayment, which calls authMiddleware"
    }
  ],
  "not_impacted": [
    {
      "entity": "generateReport()",
      "file": "src/admin/reports.ts",
      "confidence_safe": 0.90,
      "reason": "No graph path exists to changed entities"
    }
  ],
  "stats": {
    "total_entities": 7,
    "direct": 2,
    "indirect": 3,
    "not_impacted": 1,
    "max_depth": 3
  }
}
```

### Reading the Output

- **Direct impacts** are one hop away from the changed entity. `authMiddleware()` imports `validateToken()` with 0.95 edge confidence, yielding 94.9% impact confidence. The JIRA requirement has a weaker link (0.70), reflecting that requirement-to-code traceability is inherently less certain than code-to-code imports.

- **Indirect impacts** are reached by traversing multiple edges. Each hop multiplies confidences, so values decay the further you get from the source. `POST /api/checkout` at 3 hops lands at 68.9% -- still above the default test threshold.

- **Not impacted** entities have no graph path to the change. `generateReport()` is structurally isolated.

---

## 4. Test Intent

Output from `POST /intent`, cleaned up for readability.

### Response JSON

```json
{
  "high_confidence": [
    {
      "intent": "Verify authMiddleware() still functions correctly",
      "confidence": 0.949,
      "target": "authMiddleware()",
      "rationale": "Direct import dependency on changed validateToken()"
    },
    {
      "intent": "Verify authMiddleware() error handling",
      "confidence": 0.854,
      "target": "authMiddleware()",
      "rationale": "Return type change introduces new error variant (AuthError)"
    },
    {
      "intent": "Verify JIRA-AUTH-42 error handling",
      "confidence": 0.625,
      "target": "JIRA-AUTH-42",
      "rationale": "Requirement depends on validateToken behavior contract"
    }
  ],
  "medium_confidence": [
    {
      "intent": "Verify JIRA-AUTH-42 still functions",
      "confidence": 0.694,
      "target": "JIRA-AUTH-42"
    },
    {
      "intent": "Verify getUserProfile() E2E flow",
      "confidence": 0.802,
      "hops": 2,
      "target": "getUserProfile()"
    },
    {
      "intent": "Verify processPayment() E2E flow",
      "confidence": 0.755,
      "hops": 2,
      "target": "processPayment()"
    },
    {
      "intent": "Verify POST /api/checkout E2E flow",
      "confidence": 0.689,
      "hops": 3,
      "target": "POST /api/checkout"
    }
  ],
  "risks": [
    {
      "severity": "HIGH",
      "entity": "authMiddleware()",
      "reason": "Directly depends on changed code"
    },
    {
      "severity": "HIGH",
      "entity": "JIRA-AUTH-42",
      "reason": "Directly depends on changed code"
    },
    {
      "severity": "MEDIUM",
      "entity": "getUserProfile()",
      "reason": "Indirectly affected via 2-hop chain (80%)"
    },
    {
      "severity": "MEDIUM",
      "entity": "processPayment()",
      "reason": "Indirectly affected via 2-hop chain (76%)"
    }
  ],
  "coverage_gaps": "All 5 impacted entities have no linked test coverage",
  "not_tested": [
    {
      "entity": "generateReport()",
      "reason": "No graph path exists to changed entities",
      "confidence_safe": 0.90
    }
  ],
  "summary": {
    "total_scenarios": 7,
    "high": 3,
    "medium": 4,
    "low": 0,
    "risks": 4
  }
}
```

### How Test Intent Maps to Impact

The system does not just mirror the impact list. It generates distinct intents per impacted entity:

- **Functional verification** -- does the entity still produce correct output?
- **Error handling** -- does the entity handle the new failure modes introduced by the change?
- **E2E flow** -- for multi-hop impacts, does the full call chain still behave end-to-end?

High-confidence intents (authMiddleware functional + error handling) get two separate test scenarios because the return type change introduces both a new success shape and a new error shape.

---

## 5. What We Decided NOT to Test

### generateReport() -- src/admin/reports.ts

**Decision:** Do not test. 90% confidence of safety.

**Reasoning chain:**

1. The system performed a full graph traversal from the changed entity (`validateToken()`) outward through all edges.
2. `generateReport()` has zero incoming or outgoing edges connecting it to any entity in the impact subgraph.
3. It is a synchronous function with zero parameters -- it takes no auth context, no user context, and no payment context.
4. There is no transitive path of any length from `validateToken()` to `generateReport()`.

**Why not 100% safe?** The 90% figure (rather than 100%) accounts for the possibility of implicit dependencies not captured in the graph -- for example, shared global state, database tables, or runtime configuration that both entities read. The graph models explicit code-level relationships; it cannot guarantee the absence of coupling through side channels.

**Practical implication:** A team reviewing this output can skip `generateReport()` with high confidence, but should remain aware that graph completeness is an assumption, not a guarantee.

---

## 6. Explanation Chain Deep Dive

### processPayment() -- confidence 0.755, 2 hops

The system computes impact confidence by multiplying edge weights along the shortest path from the changed entity to the target.

**Path:** `validateToken()` --> `authMiddleware()` --> `processPayment()`

**Step-by-step calculation:**

```
Step 1: Start at the changed entity
  validateToken()
  Base confidence: 1.0 (this is the source of the change)

Step 2: Traverse edge -- authMiddleware imports validateToken
  Edge type:       Imports
  Edge confidence: 0.949
  Cumulative:      1.0 x 0.949 = 0.949

Step 3: Traverse edge -- processPayment calls authMiddleware
  Edge type:       Calls
  Edge confidence: 0.795
  Cumulative:      0.949 x 0.795 = 0.754455

Step 4: Round to three decimal places
  Final confidence: 0.755
```

**Why the Calls edge is 0.795 and not 0.80:**
The raw edge weight in the graph is 0.80. Temporal decay and context adjustments can shift this slightly. The system applies a small decay factor based on how recently the call relationship was confirmed by a code scan. In this demo the effective weight after adjustment lands at approximately 0.795, yielding the observed 0.755 final confidence.

**Interpretation:** There is a 75.5% chance that a breaking change to `validateToken()` will require changes to `processPayment()`. This is high enough to warrant a medium-priority test scenario but not high enough to flag as a critical direct dependency.

### Contrast: POST /api/checkout -- confidence 0.689, 3 hops

```
Path: validateToken() --> authMiddleware() --> processPayment() --> POST /api/checkout

  1.0 x 0.949 x 0.795 x 0.913 = 0.689
           ^       ^       ^
           |       |       +-- Implements edge (0.92, decayed to ~0.913)
           |       +---------- Calls edge (0.80, decayed to ~0.795)
           +------------------ Imports edge (0.95, decayed to ~0.949)
```

Each additional hop compounds the uncertainty. By the third hop, the system is less than 70% confident -- still actionable, but lower priority than the 2-hop entities.

---

## 7. Confidence Model in Action

### Noisy-OR Aggregation

When multiple independent paths exist from a changed entity to a target, the system uses the Noisy-OR model to combine them. In this demo graph, each impacted entity has exactly one path, so Noisy-OR reduces to simple multiplication. But consider a hypothetical scenario:

```
Suppose getUserProfile() also had a direct Imports edge to validateToken()
with confidence 0.60.

Path A (direct):   0.60
Path B (indirect): 0.949 x 0.85 = 0.807

Noisy-OR combination:
  P(impacted) = 1 - (1 - 0.60) x (1 - 0.807)
              = 1 - 0.40 x 0.193
              = 1 - 0.0772
              = 0.923
```

The Noisy-OR formula captures the intuition that if either path carries the impact, the entity is affected. Two weak signals reinforce each other into a strong one.

### Temporal Decay

Edge confidences are not static. They decay over time based on when the relationship was last confirmed by a code scan or commit analysis.

```
effective_confidence = base_confidence x decay_factor
decay_factor         = exp(-lambda x days_since_last_scan)
```

In this demo, the decay is minimal (all edges were recently scanned), which is why the observed confidences are close to the raw edge weights. In a real codebase with stale dependency data, temporal decay can significantly reduce confidence on edges that have not been re-validated, pushing borderline entities below the test threshold.

### Decision Thresholds

The system applies configurable thresholds to partition test intents:

| Tier   | Confidence Range | Action                      |
|--------|------------------|-----------------------------|
| High   | >= 0.80          | Must test                   |
| Medium | 0.50 -- 0.79     | Recommended                 |
| Low    | 0.30 -- 0.49     | Optional, risk-based        |
| Skip   | < 0.30           | Do not test (with caveat)   |

In this scenario, no entity falls below 0.50, so the system produces zero low-priority intents. The "not impacted" category (generateReport at 0.90 safe) is a separate classification -- it is not "low confidence of impact" but rather "high confidence of no impact."

---

## Summary

| Entity               | Hops | Confidence | Tier   | Test? |
|----------------------|------|------------|--------|-------|
| authMiddleware()     | 1    | 94.9%      | High   | Yes   |
| JIRA-AUTH-42         | 1    | 69.4%      | Medium | Yes   |
| getUserProfile()     | 2    | 80.2%      | High   | Yes   |
| processPayment()     | 2    | 75.5%      | Medium | Yes   |
| POST /api/checkout   | 3    | 68.9%      | Medium | Yes   |
| generateReport()     | --   | 90% safe   | Skip   | No    |

The UCM system traced a single function signature change through a 7-entity graph and produced 7 test scenarios (3 high, 4 medium), identified 4 risks, flagged a complete coverage gap across all impacted entities, and provided a reasoned justification for excluding the one entity with no dependency path to the change.
