const FAQS = [
  {
    q: "How is UCM different from RAG?",
    a: "RAG ranks by vector similarity — topical relevance, not factual reliability. UCM adds three layers on top: confidence scoring (Bayesian fusion), change tracking (graph-distance propagation), and incremental revalidation (volatility classification). It can augment an existing RAG pipeline or replace the retrieval layer entirely.",
  },
  {
    q: "Is UCM production-ready?",
    a: "UCM is a reference architecture backed by published Rust crates (v0.1.3 on crates.io). Each layer is grounded in production-validated research (Google Knowledge Vault, Meta TAP, Salsa), but the integrated pipeline has not been tested on enterprise data. We offer consulting engagements to validate and deploy UCM for your specific workloads.",
  },
  {
    q: "What does Krava add?",
    a: "Krava provides the privacy and security layer: identity-less passkey authentication (FIDO2), encrypted-at-rest memory, and agentic routing with session isolation. UCM handles knowledge reliability; Krava handles data sovereignty. Together they enable portable, private, reliable context across models and applications.",
  },
  {
    q: "How do I calibrate confidence scores for my data?",
    a: "UCM's calibration layer uses Noisy-OR Bayesian fusion — multi-source agreement compounds confidence. You configure which sources (APIs, databases, documents, tickets) feed each entity, and the system learns calibrated scores. When the output says 0.96, empirical accuracy should be 0.96.",
  },
  {
    q: "Can I use UCM with my existing LLM stack?",
    a: "Yes. UCM is model-agnostic — the Rust crates handle confidence scoring, graph traversal, and revalidation independently of the LLM. The ucm-api crate provides REST endpoints that integrate with any retrieval or generation pipeline. Krava's adapter layer supports multiple backends (Anthropic, Tinfoil, etc.).",
  },
  {
    q: "What industries is this best suited for?",
    a: "Any domain where knowledge changes at different rates and errors carry real cost: banking/compliance (policy changes affecting agent decisions), document management (contract and regulatory updates), healthcare (protocol and formulary changes), and enterprise SaaS (API contracts and feature flags).",
  },
];

export function FaqSection() {
  const jsonLd = {
    "@context": "https://schema.org",
    "@type": "FAQPage",
    mainEntity: FAQS.map((faq) => ({
      "@type": "Question",
      name: faq.q,
      acceptedAnswer: {
        "@type": "Answer",
        text: faq.a,
      },
    })),
  };

  return (
    <section id="faq" className="border-t border-border bg-surface py-20 md:py-28">
      <div className="mx-auto max-w-6xl px-6">
        <script
          type="application/ld+json"
          dangerouslySetInnerHTML={{
            __html: JSON.stringify(jsonLd).replace(/</g, "\\u003c"),
          }}
        />

        <p className="eyebrow">Frequently Asked Questions</p>
        <h2 className="mt-3 font-serif text-[clamp(28px,4vw,40px)] leading-tight text-ink">
          Common questions
        </h2>

        <div className="mt-12 divide-y divide-border">
          {FAQS.map((faq) => (
            <details key={faq.q} className="group py-5">
              <summary className="flex cursor-pointer items-center justify-between text-sm font-semibold text-ink">
                {faq.q}
                <svg
                  width="16"
                  height="16"
                  viewBox="0 0 16 16"
                  fill="none"
                  className="shrink-0 text-muted transition-transform group-open:rotate-45"
                >
                  <path
                    d="M8 3v10M3 8h10"
                    stroke="currentColor"
                    strokeWidth="1.5"
                  />
                </svg>
              </summary>
              <p className="mt-3 max-w-3xl text-sm leading-relaxed text-muted">
                {faq.a}
              </p>
            </details>
          ))}
        </div>
      </div>
    </section>
  );
}
