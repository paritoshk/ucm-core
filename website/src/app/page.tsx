import { Navbar } from "@/components/navbar";
import { Hero } from "@/components/hero";
import {
  ProblemSection,
  ApproachesSection,
  LayersSection,
  CratesSection,
  CaseStudySection,
} from "@/components/sections";
import { ContactForm } from "@/components/contact-form";
import { FaqSection } from "@/components/faq";
import { Footer } from "@/components/footer";

export default function Home() {
  return (
    <>
      <Navbar />
      <main>
        <Hero />
        <ProblemSection />
        <ApproachesSection />
        <LayersSection />
        <CratesSection />
        <CaseStudySection />

        {/* Contact form section */}
        <section id="contact" className="border-t border-border py-20 md:py-28">
          <div className="mx-auto max-w-6xl px-6">
            <div className="grid gap-12 md:grid-cols-2">
              <div>
                <p className="eyebrow">Get Started</p>
                <h2 className="mt-3 font-display text-[clamp(28px,4vw,40px)] leading-tight text-ink">
                  Book a technical consult
                </h2>
                <p className="mt-4 text-base text-ink-2">
                  We help engineering teams adopt UCM + Krava for their specific
                  workloads — from architecture review to production deployment.
                </p>
                <div className="mt-8 space-y-4">
                  {[
                    {
                      label: "Architecture Review",
                      desc: "Map UCM's three layers to your knowledge topology",
                    },
                    {
                      label: "Proof of Concept",
                      desc: "Validate confidence calibration on your data",
                    },
                    {
                      label: "Production Deployment",
                      desc: "Integrate Rust crates + Krava SDK into your stack",
                    },
                  ].map((item) => (
                    <div
                      key={item.label}
                      className="flex gap-3 rounded-[2px] border border-border bg-surface p-4"
                    >
                      <span className="mt-0.5 flex h-5 w-5 shrink-0 items-center justify-center rounded-full bg-amber-tint">
                        <span className="h-1.5 w-1.5 rounded-full bg-amber" />
                      </span>
                      <div>
                        <p className="text-sm font-semibold text-ink">
                          {item.label}
                        </p>
                        <p className="text-[13px] text-muted">{item.desc}</p>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
              <div>
                <ContactForm />
              </div>
            </div>
          </div>
        </section>

        <FaqSection />
      </main>
      <Footer />
    </>
  );
}
