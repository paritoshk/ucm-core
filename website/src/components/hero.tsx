"use client";

import { useState } from "react";

const PERSONAS = [
  {
    id: "cto",
    label: "CTO / VP Eng",
    headline: "Reliable knowledge\nfor agentic AI",
    sub: "Three orthogonal layers — calibrated trust, change awareness, targeted revalidation — composed into one reference architecture. Open-source Rust crates. Production-validated research foundations.",
  },
  {
    id: "ciso",
    label: "CISO / Security",
    headline: "Privacy-sovereign\nmemory for agents",
    sub: "Encrypted-at-rest context, identity-less passkey auth, and portable memory across models. UCM + Krava: own your agents' knowledge without vendor lock-in or data exposure.",
  },
  {
    id: "founder",
    label: "Founder / Platform",
    headline: "Ship agentic features\nwithout the context tax",
    sub: "Drop-in Rust crates for confidence scoring, impact propagation, and incremental revalidation. Stop rebuilding stale knowledge pipelines — let the graph track what changed.",
  },
] as const;

const KPIS = [
  { value: "42%", label: "of companies abandoned AI initiatives (S&P 2025)", delta: "↑ from 21%" },
  { value: "0.96", label: "calibrated accuracy via multi-source fusion (KV)", delta: "Bayesian" },
  { value: "33%", label: "of tests catch 99.9% of failures (Meta TAP)", delta: "graph distance" },
  { value: "~0ms", label: "revalidation for 90%+ queries (Salsa)", delta: "incremental" },
];

const CLI_LINES = [
  { prompt: "# Scan a TypeScript project", cmd: "" },
  { prompt: "$", cmd: " ucm scan ./src --language typescript" },
  { prompt: "", cmd: "" },
  { prompt: "# What breaks if I change validateToken?", cmd: "" },
  { prompt: "$", cmd: " ucm impact src/auth/service.ts validateToken" },
  { prompt: "", cmd: "" },
  { prompt: "# Get test recommendations", cmd: "" },
  { prompt: "$", cmd: " ucm intent src/auth/service.ts validateToken" },
];

export function Hero() {
  const [active, setActive] = useState(0);
  const [showCli, setShowCli] = useState(false);
  const persona = PERSONAS[active];

  return (
    <section className="relative overflow-hidden pb-16 pt-28 md:pb-24 md:pt-36">
      <div className="mx-auto max-w-6xl px-6">
        {/* Persona selector */}
        <div className="mb-8 flex items-center gap-1 rounded-[2px] border border-border bg-surface p-0.5 md:w-fit">
          {PERSONAS.map((p, i) => (
            <button
              key={p.id}
              onClick={() => setActive(i)}
              className={`rounded-[2px] px-4 py-1.5 text-[13px] font-medium font-sans transition-all ${
                active === i
                  ? "bg-canvas text-ink shadow-[0_1px_2px_rgba(28,25,23,0.08)]"
                  : "text-muted hover:text-ink"
              }`}
            >
              {p.label}
            </button>
          ))}
        </div>

        {/* Headline */}
        <h1 className="font-display text-[clamp(40px,6vw,72px)] leading-[1.05] tracking-tight text-ink whitespace-pre-line">
          {persona.headline}
        </h1>

        <p className="mt-6 max-w-2xl text-base leading-relaxed text-ink-2 md:text-lg">
          {persona.sub}
        </p>

        {/* Showcase buttons */}
        <div className="mt-8 flex flex-wrap gap-3">
          <a
            href="https://github.com/paritoshk/ucm-core"
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center justify-center gap-2 rounded-[2px] bg-ink px-6 py-2.5 text-sm font-semibold font-sans text-canvas transition-colors hover:bg-ink-2"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0016 8c0-4.42-3.58-8-8-8z" />
            </svg>
            View on GitHub
          </a>
          <a
            href="https://crates.io/users/paritoshk"
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center justify-center gap-2 rounded-[2px] bg-amber px-6 py-2.5 text-sm font-semibold font-sans text-white transition-colors hover:bg-amber-2"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <path d="M3 1h10a2 2 0 012 2v10a2 2 0 01-2 2H3a2 2 0 01-2-2V3a2 2 0 012-2zm1.5 3a.5.5 0 00-.5.5v7a.5.5 0 00.5.5h7a.5.5 0 00.5-.5v-7a.5.5 0 00-.5-.5h-7zM6 6h4v1.5H6V6zm0 2.5h4V10H6V8.5z" />
            </svg>
            Browse Crates
          </a>
          <a
            href="#contact"
            className="inline-flex items-center justify-center rounded-[2px] border border-border px-6 py-2.5 text-sm font-semibold font-sans text-ink transition-colors hover:bg-surface"
          >
            Book a Technical Consult
          </a>
          <a
            href="#layers"
            className="inline-flex items-center justify-center rounded-[2px] border border-border px-6 py-2.5 text-sm font-semibold font-sans text-ink transition-colors hover:bg-surface"
          >
            Read the Architecture
          </a>
        </div>

        {/* KPI cards */}
        <div className="mt-14 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
          {KPIS.map((kpi) => (
            <div
              key={kpi.value}
              className="rounded-[2px] border border-border bg-canvas p-5"
            >
              <div className="flex items-baseline gap-2">
                <span className="font-mono text-[32px] font-medium leading-none text-ink">
                  {kpi.value}
                </span>
                <span className="rounded-full bg-amber-tint px-2 py-0.5 font-mono text-[10px] font-medium text-amber-ink">
                  {kpi.delta}
                </span>
              </div>
              <p className="mt-2 text-[13px] leading-snug text-muted">
                {kpi.label}
              </p>
            </div>
          ))}
        </div>

        {/* CLI toggle */}
        <div className="mt-10">
          <button
            onClick={() => setShowCli(!showCli)}
            className="inline-flex items-center gap-2 rounded-[2px] border border-border bg-surface px-4 py-2 text-[13px] font-medium font-sans text-muted transition-colors hover:text-ink hover:bg-surface-2"
          >
            <svg
              width="14"
              height="14"
              viewBox="0 0 14 14"
              fill="none"
              className="opacity-60"
            >
              <path d="M2 3l4 4-4 4" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
              <path d="M8 11h4" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
            </svg>
            {showCli ? "Hide" : "Show"} CLI Quick Start
            <svg
              width="10"
              height="10"
              viewBox="0 0 10 10"
              fill="none"
              className={`transition-transform ${showCli ? "rotate-180" : ""}`}
            >
              <path d="M2 3.5l3 3 3-3" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round" />
            </svg>
          </button>

          {showCli && (
            <div className="mt-3 overflow-hidden rounded-[2px] border border-white/10 bg-rail">
              <div className="flex items-center gap-2 border-b border-white/10 px-4 py-2">
                <span className="h-2.5 w-2.5 rounded-full bg-crit/60" />
                <span className="h-2.5 w-2.5 rounded-full bg-amber/60" />
                <span className="h-2.5 w-2.5 rounded-full bg-ok/60" />
                <span className="ml-2 font-mono text-[11px] text-rail-muted">terminal</span>
              </div>
              <div className="p-4 font-mono text-[13px] leading-relaxed">
                {CLI_LINES.map((line, i) => (
                  <div key={i} className={line.cmd === "" && line.prompt !== "$" ? "text-rail-muted" : ""}>
                    {line.prompt === "$" ? (
                      <>
                        <span className="text-ok">$</span>
                        <span className="text-rail-fg">{line.cmd}</span>
                      </>
                    ) : line.prompt ? (
                      <span className="text-rail-muted">{line.prompt}</span>
                    ) : (
                      <br />
                    )}
                  </div>
                ))}
              </div>
              <div className="border-t border-white/10 px-4 py-2">
                <code className="font-mono text-[11px] text-rail-muted">
                  cargo install ucm
                </code>
              </div>
            </div>
          )}
        </div>
      </div>
    </section>
  );
}
