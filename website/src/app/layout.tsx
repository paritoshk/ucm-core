import type { Metadata } from "next";
import { Inter, JetBrains_Mono, Instrument_Serif } from "next/font/google";
import "./globals.css";

const inter = Inter({
  variable: "--font-sans",
  subsets: ["latin"],
  display: "swap",
});

const jetbrains = JetBrains_Mono({
  variable: "--font-mono",
  subsets: ["latin"],
  display: "swap",
});

const instrumentSerif = Instrument_Serif({
  variable: "--font-serif",
  subsets: ["latin"],
  weight: "400",
  display: "swap",
});

export const metadata: Metadata = {
  title: "UCM — Unified Context Management | Reliable Knowledge for Agentic AI",
  description:
    "A reference architecture for calibrated trust, change awareness, and targeted revalidation in enterprise AI agents. Published Rust crates. Backed by Google Knowledge Vault, Meta TAP, and Salsa research.",
  keywords: [
    "unified context management",
    "UCM",
    "agentic AI",
    "calibrated trust",
    "change awareness",
    "targeted revalidation",
    "knowledge reliability",
    "RAG alternative",
    "enterprise AI",
    "Krava",
    "privacy SDK",
    "portable context",
  ],
  authors: [{ name: "Paritosh Kulkarni" }],
  openGraph: {
    title: "UCM — Reliable Knowledge for Agentic AI",
    description:
      "Three orthogonal layers — calibrated trust, change awareness, targeted revalidation — composed into one framework. Open-source Rust crates + enterprise consulting.",
    type: "website",
    locale: "en_US",
    siteName: "UCM by Attian AI",
  },
  twitter: {
    card: "summary_large_image",
    title: "UCM — Reliable Knowledge for Agentic AI",
    description:
      "Three orthogonal layers — calibrated trust, change awareness, targeted revalidation — composed into one framework.",
  },
  robots: {
    index: true,
    follow: true,
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  const jsonLd = {
    "@context": "https://schema.org",
    "@type": "Organization",
    name: "Attian AI",
    description:
      "Unified Context Management — a reference architecture for reliable knowledge in enterprise agentic AI.",
    url: "https://ucm-core.vercel.app",
    sameAs: [
      "https://github.com/paritoshk/ucm-core",
      "https://crates.io/users/paritoshk",
    ],
  };

  return (
    <html
      lang="en"
      className={`${inter.variable} ${jetbrains.variable} ${instrumentSerif.variable} h-full`}
    >
      <body className="min-h-full">
        <script
          type="application/ld+json"
          dangerouslySetInnerHTML={{
            __html: JSON.stringify(jsonLd).replace(/</g, "\\u003c"),
          }}
        />
        {children}
      </body>
    </html>
  );
}
