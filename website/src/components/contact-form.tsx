"use client";

import { useState, type FormEvent } from "react";

const ROLE_OPTIONS = [
  "CTO / VP Engineering",
  "CISO / Security",
  "Founder / CEO",
  "Staff+ Engineer",
  "Other",
];

const INTEREST_OPTIONS = [
  "UCM — Knowledge Reliability",
  "Krava — Privacy SDK",
  "Both (bundled)",
  "Not sure yet",
];

const SIZE_OPTIONS = ["1–10", "11–50", "51–200", "200+"];
const TIMELINE_OPTIONS = ["Now", "This quarter", "Exploring"];

type FormStatus = "idle" | "submitting" | "success" | "error";

export function ContactForm() {
  const [status, setStatus] = useState<FormStatus>("idle");
  const [errorMsg, setErrorMsg] = useState("");

  async function handleSubmit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setStatus("submitting");
    setErrorMsg("");

    const form = e.currentTarget;
    const data = Object.fromEntries(new FormData(form));

    try {
      const res = await fetch("/api/contact", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(data),
      });

      if (!res.ok) {
        const body = await res.json().catch(() => ({}));
        throw new Error(body.error || "Something went wrong");
      }

      setStatus("success");
      form.reset();
    } catch (err) {
      setStatus("error");
      setErrorMsg(err instanceof Error ? err.message : "Submission failed");
    }
  }

  if (status === "success") {
    return (
      <div className="rounded-[2px] border border-ok/30 bg-ok-tint p-8 text-center">
        <p className="font-display text-2xl text-ink">Thank you</p>
        <p className="mt-2 text-sm text-muted">
          We&apos;ll be in touch within one business day.
        </p>
        <button
          onClick={() => setStatus("idle")}
          className="mt-4 text-sm font-medium text-amber underline underline-offset-2"
        >
          Submit another inquiry
        </button>
      </div>
    );
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-5">
      {/* Name + Email */}
      <div className="grid gap-4 sm:grid-cols-2">
        <div>
          <label htmlFor="name" className="text-xs font-medium text-ink-2">
            Full name <span className="text-amber">*</span>
          </label>
          <input
            id="name"
            name="name"
            type="text"
            required
            className="mt-1 block w-full rounded-[2px] border border-border bg-canvas px-3 py-2 text-sm text-ink placeholder:text-muted-2 focus:border-amber focus:outline-none focus:ring-1 focus:ring-amber"
            placeholder="Jane Doe"
          />
        </div>
        <div>
          <label htmlFor="email" className="text-xs font-medium text-ink-2">
            Work email <span className="text-amber">*</span>
          </label>
          <input
            id="email"
            name="email"
            type="email"
            required
            className="mt-1 block w-full rounded-[2px] border border-border bg-canvas px-3 py-2 text-sm text-ink placeholder:text-muted-2 focus:border-amber focus:outline-none focus:ring-1 focus:ring-amber"
            placeholder="jane@company.com"
          />
        </div>
      </div>

      {/* Company + Role */}
      <div className="grid gap-4 sm:grid-cols-2">
        <div>
          <label htmlFor="company" className="text-xs font-medium text-ink-2">
            Company <span className="text-amber">*</span>
          </label>
          <input
            id="company"
            name="company"
            type="text"
            required
            className="mt-1 block w-full rounded-[2px] border border-border bg-canvas px-3 py-2 text-sm text-ink placeholder:text-muted-2 focus:border-amber focus:outline-none focus:ring-1 focus:ring-amber"
            placeholder="Acme Corp"
          />
        </div>
        <div>
          <label htmlFor="role" className="text-xs font-medium text-ink-2">
            Role <span className="text-amber">*</span>
          </label>
          <select
            id="role"
            name="role"
            required
            className="mt-1 block w-full rounded-[2px] border border-border bg-canvas px-3 py-2 text-sm text-ink focus:border-amber focus:outline-none focus:ring-1 focus:ring-amber"
          >
            <option value="">Select your role</option>
            {ROLE_OPTIONS.map((r) => (
              <option key={r} value={r}>
                {r}
              </option>
            ))}
          </select>
        </div>
      </div>

      {/* Use case */}
      <div>
        <label htmlFor="useCase" className="text-xs font-medium text-ink-2">
          What are you building? <span className="text-amber">*</span>
        </label>
        <textarea
          id="useCase"
          name="useCase"
          required
          rows={3}
          className="mt-1 block w-full rounded-[2px] border border-border bg-canvas px-3 py-2 text-sm text-ink placeholder:text-muted-2 focus:border-amber focus:outline-none focus:ring-1 focus:ring-amber"
          placeholder="Describe your use case — agents, documents, compliance, etc."
        />
      </div>

      {/* Interest + Size + Timeline */}
      <div className="grid gap-4 sm:grid-cols-3">
        <div>
          <label htmlFor="interest" className="text-xs font-medium text-ink-2">
            Interest <span className="text-amber">*</span>
          </label>
          <select
            id="interest"
            name="interest"
            required
            className="mt-1 block w-full rounded-[2px] border border-border bg-canvas px-3 py-2 text-sm text-ink focus:border-amber focus:outline-none focus:ring-1 focus:ring-amber"
          >
            <option value="">Select</option>
            {INTEREST_OPTIONS.map((o) => (
              <option key={o} value={o}>
                {o}
              </option>
            ))}
          </select>
        </div>
        <div>
          <label htmlFor="size" className="text-xs font-medium text-ink-2">
            Company size
          </label>
          <select
            id="size"
            name="size"
            className="mt-1 block w-full rounded-[2px] border border-border bg-canvas px-3 py-2 text-sm text-ink focus:border-amber focus:outline-none focus:ring-1 focus:ring-amber"
          >
            <option value="">Select</option>
            {SIZE_OPTIONS.map((o) => (
              <option key={o} value={o}>
                {o}
              </option>
            ))}
          </select>
        </div>
        <div>
          <label htmlFor="timeline" className="text-xs font-medium text-ink-2">
            Timeline
          </label>
          <select
            id="timeline"
            name="timeline"
            className="mt-1 block w-full rounded-[2px] border border-border bg-canvas px-3 py-2 text-sm text-ink focus:border-amber focus:outline-none focus:ring-1 focus:ring-amber"
          >
            <option value="">Select</option>
            {TIMELINE_OPTIONS.map((o) => (
              <option key={o} value={o}>
                {o}
              </option>
            ))}
          </select>
        </div>
      </div>

      {status === "error" && (
        <p className="text-sm text-crit">{errorMsg}</p>
      )}

      <button
        type="submit"
        disabled={status === "submitting"}
        className="w-full rounded-[2px] bg-amber px-6 py-3 text-sm font-semibold text-white transition-colors hover:bg-amber-2 disabled:opacity-60"
      >
        {status === "submitting" ? "Sending…" : "Request a Technical Consult"}
      </button>

      <p className="text-center text-xs text-muted-2">
        We&apos;ll respond within one business day. No spam — ever.
      </p>
    </form>
  );
}
