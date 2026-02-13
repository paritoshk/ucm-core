import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { ArchitectureView } from "@/components/architecture-view"
import { DemoView } from "@/components/demo-view"
import { DataFlowView } from "@/components/data-flow-view"
import { ImpactSimulator } from "@/components/impact-simulator"

function App() {
  return (
    <div className="min-h-screen bg-background text-foreground">
      {/* Hero */}
      <header className="relative overflow-hidden border-b border-border/40 bg-gradient-to-b from-zinc-900 via-zinc-950 to-background">
        <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_top,rgba(120,119,198,0.12),transparent_50%)]" />
        <div className="relative mx-auto max-w-6xl px-6 py-16 text-center">
          <div className="mb-3 inline-flex items-center gap-2 rounded-full border border-border/60 bg-zinc-800/60 px-4 py-1.5 text-xs font-medium text-zinc-300 backdrop-blur-sm">
            <span className="inline-block h-2 w-2 rounded-full bg-emerald-400 animate-pulse" />
            Context Intelligence System
          </div>
          <h1 className="text-4xl font-bold tracking-tight text-zinc-50 sm:text-5xl lg:text-6xl">
            Context<span className="text-transparent bg-clip-text bg-gradient-to-r from-violet-400 to-cyan-400">QA</span>
          </h1>
          <p className="mx-auto mt-4 max-w-2xl text-lg text-zinc-400">
            Rust core &middot; Event sourcing &middot; Probabilistic knowledge graph &middot; Impact reasoning
          </p>
          <div className="mt-6 flex flex-wrap items-center justify-center gap-2">
            <span className="rounded-md border border-violet-500/30 bg-violet-500/10 px-3 py-1 text-xs font-medium text-violet-300">
              Replayable decisions
            </span>
            <span className="rounded-md border border-cyan-500/30 bg-cyan-500/10 px-3 py-1 text-xs font-medium text-cyan-300">
              Bayesian confidence
            </span>
            <span className="rounded-md border border-amber-500/30 bg-amber-500/10 px-3 py-1 text-xs font-medium text-amber-300">
              SCIP identity
            </span>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="mx-auto max-w-6xl px-6 py-10">
        <Tabs defaultValue="architecture" className="w-full">
          <TabsList className="mb-8 grid w-full grid-cols-4 h-12">
            <TabsTrigger value="architecture" className="text-sm font-medium">Architecture</TabsTrigger>
            <TabsTrigger value="demo" className="text-sm font-medium">Demo &amp; Reasoning</TabsTrigger>
            <TabsTrigger value="dataflow" className="text-sm font-medium">Data Flow</TabsTrigger>
            <TabsTrigger value="simulator" className="text-sm font-medium">Impact Simulator</TabsTrigger>
          </TabsList>
          <TabsContent value="architecture"><ArchitectureView /></TabsContent>
          <TabsContent value="demo"><DemoView /></TabsContent>
          <TabsContent value="dataflow"><DataFlowView /></TabsContent>
          <TabsContent value="simulator"><ImpactSimulator /></TabsContent>
        </Tabs>
      </main>

      {/* Footer */}
      <footer className="border-t border-border/40 py-8 text-center text-xs text-zinc-500">
        ContextQA &middot; Built with Rust + petgraph + Axum &middot; Dashboard: Vite + React + shadcn/ui
      </footer>
    </div>
  )
}

export default App
