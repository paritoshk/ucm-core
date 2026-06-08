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

export function Hero() {
  const [active, setActive] = useState(0);
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
              className={`rounded-[2px] px-4 py-1.5 text-[13px] font-medium transition-all ${
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
        <h1 className="font-serif text-[clamp(40px,6vw,72px)] leading-[1.05] tracking-tight text-ink whitespace-pre-line">
          {persona.headline}
        </h1>

        <p className="mt-6 max-w-2xl text-base leading-relaxed text-ink-2 md:text-lg">
          {persona.sub}
        </p>

        {/* CTAs */}
        <div className="mt-8 flex flex-col gap-3 sm:flex-row">
          <a
            href="#contact"
            className="inline-flex items-center justify-center rounded-[2px] bg-amber px-6 py-2.5 text-sm font-semibold text-white transition-colors hover:bg-amber-2"
          >
            Book a Technical Consult
          </a>
          <a
            href="#layers"
            className="inline-flex items-center justify-center rounded-[2px] border border-border px-6 py-2.5 text-sm font-semibold text-ink transition-colors hover:bg-surface"
          >
            Read the Architecture
          </a>
          <a
            href="https://krava.io"
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center justify-center gap-1.5 rounded-[2px] border border-border px-6 py-2.5 text-sm font-semibold text-ink transition-colors hover:bg-surface"
          >
            Explore Krava
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none" className="opacity-50">
              <path d="M3.5 2h6.5v6.5M9.5 2.5L2 10" stroke="currentColor" strokeWidth="1.2" />
            </svg>
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
      </div>
    </section>
  );
}
