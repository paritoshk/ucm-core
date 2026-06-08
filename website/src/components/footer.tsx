export function Footer() {
  return (
    <footer className="border-t border-border bg-rail py-12">
      <div className="mx-auto max-w-6xl px-6">
        <div className="grid gap-8 sm:grid-cols-3">
          {/* Brand */}
          <div>
            <span className="font-serif text-xl text-rail-fg">UCM</span>
            <p className="mt-2 text-sm text-rail-muted">
              Unified Context Management — a reference architecture for reliable
              knowledge in agentic AI.
            </p>
            <p className="mt-3 text-xs text-rail-muted">
              Kulkarni, Shaw, El Bouri · Krava · 2026
            </p>
          </div>

          {/* Resources */}
          <div>
            <p className="text-xs font-semibold uppercase tracking-wider text-rail-muted">
              Resources
            </p>
            <ul className="mt-3 space-y-2 text-sm text-rail-fg/70">
              <li>
                <a
                  href="https://github.com/paritoshk/ucm-core"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="hover:text-rail-fg"
                >
                  GitHub Repository
                </a>
              </li>
              <li>
                <a
                  href="https://crates.io/users/paritoshk"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="hover:text-rail-fg"
                >
                  crates.io Profile
                </a>
              </li>
              <li>
                <a
                  href="https://docs.rs/ucm-graph-core"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="hover:text-rail-fg"
                >
                  API Documentation
                </a>
              </li>
              <li>
                <a
                  href="https://ucm-fcmh4v1.gamma.site/"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="hover:text-rail-fg"
                >
                  Presentation
                </a>
              </li>
            </ul>
          </div>

          {/* Partners */}
          <div>
            <p className="text-xs font-semibold uppercase tracking-wider text-rail-muted">
              Partners
            </p>
            <ul className="mt-3 space-y-2 text-sm text-rail-fg/70">
              <li>
                <a
                  href="https://krava.io"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="hover:text-rail-fg"
                >
                  Krava — Privacy SDK
                </a>
              </li>
              <li>
                <a href="#contact" className="hover:text-rail-fg">
                  Contact / Consulting
                </a>
              </li>
            </ul>
          </div>
        </div>

        <div className="mt-10 border-t border-white/10 pt-6 text-center text-xs text-rail-muted">
          © {new Date().getFullYear()} Krava. UCM is a design specification
          — see{" "}
          <a href="#faq" className="underline underline-offset-2 hover:text-rail-fg">
            FAQ
          </a>{" "}
          for current status.
        </div>
      </div>
    </footer>
  );
}
