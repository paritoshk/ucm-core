/* Server Components — content-only sections */

/* ── The Problem ── */
export function ProblemSection() {
  return (
    <section id="problem" className="border-t border-border py-20 md:py-28">
      <div className="mx-auto max-w-6xl px-6">
        <p className="eyebrow">The Problem</p>
        <h2 className="mt-3 font-serif text-[clamp(28px,4vw,40px)] leading-tight text-ink">
          Models work. The knowledge they operate on does not.
        </h2>
        <p className="mt-4 max-w-2xl text-base text-ink-2">
          Enterprise knowledge degrades silently: facts go stale at different
          rates, contradictions accumulate across sources, and when something
          changes, nothing indicates which downstream decisions are now
          unreliable.
        </p>

        <div className="mt-12 grid gap-6 sm:grid-cols-3">
          {[
            {
              num: "01",
              title: "Calibrated Trust",
              desc: "Every fact must carry a confidence score grounded in evidence — not just vector similarity.",
            },
            {
              num: "02",
              title: "Change Awareness",
              desc: "When a fact changes, the system must know what else is affected across the dependency graph.",
            },
            {
              num: "03",
              title: "Targeted Revalidation",
              desc: "Maintenance must focus on what actually changed — not reprocess everything on a fixed schedule.",
            },
          ].map((prop) => (
            <div
              key={prop.num}
              className="rounded-[2px] border border-border bg-surface p-6"
            >
              <span className="font-mono text-xs text-muted">{prop.num}</span>
              <h3 className="mt-2 text-base font-semibold text-ink">
                {prop.title}
              </h3>
              <p className="mt-2 text-sm leading-relaxed text-muted">
                {prop.desc}
              </p>
            </div>
          ))}
        </div>

        <p className="mt-8 text-sm text-muted">
          No current approach delivers all three simultaneously.
        </p>
      </div>
    </section>
  );
}

/* ── Why Current Approaches Fail ── */
export function ApproachesSection() {
  const fails = [
    {
      title: "RAG",
      problem: "Ranks by vector similarity — topical relevance, not factual reliability",
      detail:
        "A three-year-old code comment and yesterday's API spec score identically. No mechanism for confidence calibration, staleness detection, or selective revalidation.",
    },
    {
      title: "Scheduled Rebuilds",
      problem: "Uniform cadence applied to non-uniform change",
      detail:
        "Google TAP (2B LOC, ~1 commit/sec): 50% of targets changed <14 times/month; a volatile minority changed hundreds of times. Nightly rebuilds reprocess the stable majority and miss the volatile subset.",
    },
    {
      title: "Long Context Windows",
      problem: "More tokens = more sources of confidently wrong answers",
      detail:
        "Models ignore mid-context information (\"lost in the middle\"). TAP showed that beyond 10 dependency hops, testing produces a 99:1 noise-to-signal ratio.",
    },
  ];

  return (
    <section className="border-t border-border bg-surface py-20 md:py-28">
      <div className="mx-auto max-w-6xl px-6">
        <p className="eyebrow">Why Current Approaches Fail</p>
        <h2 className="mt-3 font-serif text-[clamp(28px,4vw,40px)] leading-tight text-ink">
          Three patterns, three failure modes
        </h2>

        <div className="mt-12 grid gap-6 md:grid-cols-3">
          {fails.map((f) => (
            <div
              key={f.title}
              className="rounded-[2px] border border-border bg-canvas p-6"
            >
              <h3 className="text-lg font-semibold text-ink">{f.title}</h3>
              <p className="mt-2 text-sm font-medium text-amber">
                {f.problem}
              </p>
              <p className="mt-3 text-sm leading-relaxed text-muted">
                {f.detail}
              </p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

/* ── The 3 Layers ── */
export function LayersSection() {
  const layers = [
    {
      num: "Layer 1",
      title: "Calibrated Trust",
      foundation: "Google Knowledge Vault (KDD 2014)",
      math: "Noisy-OR Bayesian fusion",
      how: "Extracted 1.6B facts from the web using four independent methods. Multi-source agreement compounds confidence. When the system outputs 0.96, empirical accuracy is 0.96.",
      enterprise:
        "A claim confirmed by CRM data, an API response, a support ticket, and a Confluence page — quantify agreement weight, flag single-source claims.",
    },
    {
      num: "Layer 2",
      title: "Change Awareness",
      foundation: "Meta Predictive Test Selection (ICSE-SEIP 2019)",
      math: "Graph-distance impact propagation",
      how: "Thousands of commits/day, millions of tests. Impact attenuates predictably with dependency-graph distance: shortest-path length is the strongest failure predictor. 33% of tests caught 99.9% of real failures.",
      enterprise:
        "API contract changes → directly dependent docs take a large confidence hit; three hops away, smaller; ten hops, negligible.",
    },
    {
      num: "Layer 3",
      title: "Targeted Revalidation",
      foundation: "Salsa Engine — Rust IDE (2019+)",
      math: "Durability classification + incremental re-compute",
      how: "Every input classified by expected volatility: stdlib changes every 6 weeks (high durability), source files every few seconds (low durability). On change, only dependent computations revalidate. Result: ~300ms → ~0ms for 90%+ of queries.",
      enterprise:
        "Business rules (quarterly), API contracts (monthly), feature flags (hourly) — each with revalidation frequency matched to actual volatility.",
    },
  ];

  return (
    <section id="layers" className="border-t border-border py-20 md:py-28">
      <div className="mx-auto max-w-6xl px-6">
        <p className="eyebrow">The Architecture</p>
        <h2 className="mt-3 font-serif text-[clamp(28px,4vw,40px)] leading-tight text-ink">
          Three orthogonal layers, one composition
        </h2>
        <p className="mt-4 max-w-2xl text-base text-ink-2">
          Each layer solves a single dimension using the appropriate mathematical
          tool: Bayesian fusion, graph propagation, and durability
          classification. UCM composes them into a unified framework that
          addresses all three failure modes.
        </p>

        <div className="mt-14 space-y-8">
          {layers.map((layer) => (
            <div
              key={layer.title}
              className="rounded-[2px] border border-border bg-surface p-6 md:p-8"
            >
              <div className="flex flex-wrap items-baseline gap-3">
                <span className="eyebrow">{layer.num}</span>
                <h3 className="font-serif text-2xl text-ink">
                  {layer.title}
                </h3>
              </div>
              <div className="mt-4 grid gap-6 md:grid-cols-2">
                <div>
                  <p className="text-xs font-semibold uppercase tracking-wider text-muted">
                    Research Foundation
                  </p>
                  <p className="mt-1 text-sm text-ink-2">{layer.foundation}</p>
                  <p className="mt-1 font-mono text-xs text-amber">
                    {layer.math}
                  </p>
                  <p className="mt-3 text-sm leading-relaxed text-muted">
                    {layer.how}
                  </p>
                </div>
                <div>
                  <p className="text-xs font-semibold uppercase tracking-wider text-muted">
                    Enterprise Mapping
                  </p>
                  <p className="mt-1 text-sm leading-relaxed text-ink-2">
                    {layer.enterprise}
                  </p>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

/* ── Implementation — 7 Crates ── */
export function CratesSection() {
  const crates = [
    {
      name: "ucm-graph-core",
      layer: "Foundation",
      desc: "Entity model, SCIP identity scheme, Noisy-OR confidence math, in-memory graph",
    },
    {
      name: "ucm-events",
      layer: "Layer 2",
      desc: "Append-only event sourcing — Datomic-inspired mutation log with temporal queries",
    },
    {
      name: "ucm-ingest",
      layer: "Ingestion",
      desc: "Multi-source ingestion pipeline — Markdown, JSON, YAML with configurable extractors",
    },
    {
      name: "ucm-reason",
      layer: "Layer 1 + 2",
      desc: "Bayesian confidence fusion + graph-distance impact propagation engine",
    },
    {
      name: "ucm-observe",
      layer: "Layer 3",
      desc: "Volatility classification, staleness scoring, revalidation scheduler",
    },
    {
      name: "ucm-api",
      layer: "Interface",
      desc: "REST/gRPC API surface — query, ingest, and observe endpoints",
    },
    {
      name: "ucm-cli",
      layer: "Tooling",
      desc: "CLI for local development, graph inspection, and pipeline debugging",
    },
  ];

  return (
    <section
      id="crates"
      className="border-t border-border bg-surface py-20 md:py-28"
    >
      <div className="mx-auto max-w-6xl px-6">
        <p className="eyebrow">Implementation</p>
        <h2 className="mt-3 font-serif text-[clamp(28px,4vw,40px)] leading-tight text-ink">
          7 Rust crates, published on crates.io
        </h2>
        <p className="mt-4 max-w-2xl text-base text-ink-2">
          Each crate maps to a specific layer of the architecture. All are open
          source, tested, and documented with architecture notes and research
          references.
        </p>

        <div className="mt-12 grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
          {crates.map((c) => (
            <a
              key={c.name}
              href={`https://crates.io/crates/${c.name}`}
              target="_blank"
              rel="noopener noreferrer"
              className="group rounded-[2px] border border-border bg-canvas p-5 transition-shadow hover:shadow-[0_4px_14px_-6px_rgba(28,25,23,0.12)]"
            >
              <div className="flex items-center justify-between">
                <span className="font-mono text-sm font-medium text-ink">
                  {c.name}
                </span>
                <span className="rounded-full bg-amber-tint px-2 py-0.5 text-[10px] font-semibold text-amber-ink">
                  v0.1.3
                </span>
              </div>
              <span className="mt-1 inline-block rounded-[2px] bg-surface-2 px-1.5 py-0.5 text-[10px] font-medium text-muted">
                {c.layer}
              </span>
              <p className="mt-2 text-[13px] leading-snug text-muted group-hover:text-ink-2">
                {c.desc}
              </p>
            </a>
          ))}
        </div>
      </div>
    </section>
  );
}

/* ── Case Study ── */
export function CaseStudySection() {
  return (
    <section id="case-study" className="border-t border-border py-20 md:py-28">
      <div className="mx-auto max-w-6xl px-6">
        <p className="eyebrow">Case Study</p>
        <h2 className="mt-3 font-serif text-[clamp(28px,4vw,40px)] leading-tight text-ink">
          Banking: unauthorized refund prevention
        </h2>
        <p className="mt-4 max-w-2xl text-base text-ink-2">
          When a new compliance rule supersedes an existing refund policy, agents
          must know immediately — not after the nightly rebuild. UCM propagates
          the change through the dependency graph and revalidates only the
          affected downstream decisions.
        </p>

        <div className="mt-12 grid gap-6 md:grid-cols-3">
          {[
            {
              step: "1",
              title: "Policy change ingested",
              desc: "New compliance rule enters via ucm-ingest. The event log in ucm-events records the mutation with full provenance.",
            },
            {
              step: "2",
              title: "Impact propagated",
              desc: "ucm-reason computes graph-distance impact: refund-approval rules take a large confidence hit; indirectly related KYC docs receive a smaller adjustment.",
            },
            {
              step: "3",
              title: "Targeted revalidation",
              desc: "ucm-observe marks only the affected subgraph for revalidation. Stable subgraphs (account structures, historical transactions) are untouched.",
            },
          ].map((s) => (
            <div key={s.step} className="flex gap-4">
              <span className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-amber font-mono text-sm font-bold text-white">
                {s.step}
              </span>
              <div>
                <h3 className="text-sm font-semibold text-ink">{s.title}</h3>
                <p className="mt-1 text-[13px] leading-relaxed text-muted">
                  {s.desc}
                </p>
              </div>
            </div>
          ))}
        </div>

        {/* Krava bundle panel */}
        <div className="mt-14 rounded-[2px] border border-white/10 bg-rail p-8">
          <div className="flex flex-wrap items-center gap-3">
            <span className="font-mono text-xs uppercase tracking-widest text-amber-2">
              Krava + UCM
            </span>
            <span className="rounded-full border border-white/10 px-2 py-0.5 text-[10px] text-rail-fg">
              bundled offering
            </span>
          </div>
          <h3 className="mt-3 font-serif text-2xl text-rail-fg">
            Privacy-sovereign context for enterprise agents
          </h3>
          <p className="mt-3 max-w-2xl text-sm leading-relaxed text-rail-muted">
            UCM handles knowledge reliability. Krava handles privacy and
            security — identity-less passkey auth, encrypted-at-rest memory, and
            agentic routing with session isolation. Together, they enable
            portable, private, reliable context across models and apps.
          </p>
          <div className="mt-6 flex flex-col gap-3 sm:flex-row">
            <a
              href="#contact"
              className="inline-flex items-center justify-center rounded-[2px] bg-amber px-5 py-2 text-sm font-semibold text-white hover:bg-amber-2"
            >
              Discuss an Integration
            </a>
            <a
              href="https://krava.io"
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center justify-center gap-1.5 rounded-[2px] border border-white/20 px-5 py-2 text-sm font-semibold text-rail-fg hover:bg-white/5"
            >
              Explore Krava
              <svg
                width="12"
                height="12"
                viewBox="0 0 12 12"
                fill="none"
                className="opacity-50"
              >
                <path
                  d="M3.5 2h6.5v6.5M9.5 2.5L2 10"
                  stroke="currentColor"
                  strokeWidth="1.2"
                />
              </svg>
            </a>
          </div>
        </div>
      </div>
    </section>
  );
}
