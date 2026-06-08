"use client";

import { useState, useEffect } from "react";

const NAV_LINKS = [
  { label: "Problem", href: "#problem" },
  { label: "Architecture", href: "#layers" },
  { label: "Implementation", href: "#crates" },
  { label: "Case Study", href: "#case-study" },
  { label: "FAQ", href: "#faq" },
];

export function Navbar() {
  const [scrolled, setScrolled] = useState(false);
  const [mobileOpen, setMobileOpen] = useState(false);
  const [activeSection, setActiveSection] = useState("");

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 20);
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) {
            setActiveSection(`#${entry.target.id}`);
          }
        }
      },
      { rootMargin: "-40% 0px -50% 0px" }
    );
    const ids = NAV_LINKS.map((l) => l.href.slice(1));
    ids.forEach((id) => {
      const el = document.getElementById(id);
      if (el) observer.observe(el);
    });
    return () => observer.disconnect();
  }, []);

  return (
    <header
      className={`fixed top-0 left-0 right-0 z-50 transition-all duration-200 ${
        scrolled
          ? "bg-rail/95 backdrop-blur-sm shadow-[0_1px_0_rgba(255,255,255,0.06)]"
          : "bg-transparent"
      }`}
    >
      <nav className="mx-auto flex h-14 max-w-6xl items-center justify-between px-6">
        {/* Logo */}
        <a href="#" className="flex items-center gap-2">
          <span className={`font-serif text-xl tracking-tight ${scrolled ? "text-rail-fg" : "text-fg"}`}>
            UCM
          </span>
          <span className="live-dot" />
        </a>

        {/* Desktop links */}
        <div className="hidden items-center gap-1 md:flex">
          {NAV_LINKS.map((link) => (
            <a
              key={link.href}
              href={link.href}
              className={`relative px-3 py-1.5 text-[13px] font-medium transition-colors ${
                scrolled
                  ? activeSection === link.href
                    ? "text-rail-fg"
                    : "text-rail-muted hover:text-rail-fg"
                  : activeSection === link.href
                    ? "text-fg"
                    : "text-muted hover:text-fg"
              }`}
            >
              {link.label}
              {activeSection === link.href && (
                <span className="absolute inset-x-1 -bottom-0.5 h-0.5 rounded-full bg-amber-2" />
              )}
            </a>
          ))}
        </div>

        {/* CTA */}
        <a
          href="#contact"
          className="hidden rounded-[2px] bg-amber px-4 py-1.5 text-[13px] font-semibold text-white transition-colors hover:bg-amber-2 md:block"
        >
          Book a Consult
        </a>

        {/* Mobile toggle */}
        <button
          onClick={() => setMobileOpen(!mobileOpen)}
          className={`flex h-8 w-8 items-center justify-center md:hidden ${scrolled ? "text-rail-fg" : "text-fg"}`}
          aria-label="Toggle menu"
        >
          <svg width="18" height="18" viewBox="0 0 18 18" fill="none">
            {mobileOpen ? (
              <path
                d="M4 4l10 10M14 4L4 14"
                stroke="currentColor"
                strokeWidth="1.5"
              />
            ) : (
              <>
                <path d="M2 4h14" stroke="currentColor" strokeWidth="1.5" />
                <path d="M2 9h14" stroke="currentColor" strokeWidth="1.5" />
                <path d="M2 14h14" stroke="currentColor" strokeWidth="1.5" />
              </>
            )}
          </svg>
        </button>
      </nav>

      {/* Mobile menu */}
      {mobileOpen && (
        <div className="border-t border-white/10 bg-rail px-6 pb-4 pt-2 md:hidden">
          {NAV_LINKS.map((link) => (
            <a
              key={link.href}
              href={link.href}
              onClick={() => setMobileOpen(false)}
              className="block py-2 text-sm text-rail-fg/80 hover:text-rail-fg"
            >
              {link.label}
            </a>
          ))}
          <a
            href="#contact"
            onClick={() => setMobileOpen(false)}
            className="mt-2 block rounded-[2px] bg-amber px-4 py-2 text-center text-sm font-semibold text-white"
          >
            Book a Consult
          </a>
        </div>
      )}
    </header>
  );
}
