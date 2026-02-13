import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Separator } from "@/components/ui/separator"

const flowSteps = [
    { label: "INPUT", color: "text-blue-400", border: "border-blue-500/30", text: "Source Code, Jira Tickets, API Logs → parsers → Vec<ContextEvent>" },
    { label: "EVENT", color: "text-teal-400", border: "border-teal-500/30", text: "Vec<ContextEvent> → append-only store → Event Stream" },
    { label: "GRAPH", color: "text-emerald-400", border: "border-emerald-500/30", text: "Event Stream → StableGraph projection → Fact Layers" },
    { label: "SCORE", color: "text-emerald-400", border: "border-emerald-500/30", text: "Fact Layers → Bayesian fusion → Confidence-weighted edges" },
    { label: "QUERY", color: "text-amber-400", border: "border-amber-500/30", text: "Change Set → Reverse BFS with confidence cutoff" },
    { label: "BLAST", color: "text-amber-400", border: "border-amber-500/30", text: "Reverse BFS → Direct, Indirect, Not-Impacted zones" },
    { label: "THINK", color: "text-red-400", border: "border-red-500/30", text: "Impact zones → Risk assessment → Test intent + explanation" },
    { label: "TRACE", color: "text-purple-400", border: "border-purple-500/30", text: "Every step → DecisionTrace → Replay & diff" },
    { label: "SERVE", color: "text-cyan-400", border: "border-cyan-500/30", text: "Test intent → Axum REST or WS → Dashboard" },
]

const observability = [
    { title: "OpenTelemetry spans", text: "Instrument every ingest, query, and reasoning step." },
    { title: "Decision trace log", text: "Structured record of each inference with evidence." },
    { title: "Graph diff viewer", text: "Before/after context snapshots per event batch." },
    { title: "Replay debugger", text: "Re-derive any decision from the event log and diff results." },
    { title: "Confidence heatmap", text: "Edge staleness and decay visualization in real time." },
]

const designDecisions = [
    { title: "Event log is authoritative", text: "The graph is always rebuildable from events." },
    { title: "Early cutoff", text: "Whitespace edits do not propagate impact." },
    { title: "Noisy-OR confidence", text: "Multiple paths compound instead of multiply." },
    { title: "SCIP identity strings", text: "No graph-local IDs, files re-index independently." },
    { title: "Temporal decay", text: "Stale evidence automatically loses confidence." },
]

export function DataFlowView() {
    return (
        <div className="space-y-8">
            {/* Pipeline */}
            <Card className="border-border/40 bg-zinc-900/60">
                <CardHeader>
                    <CardTitle className="text-lg text-zinc-200">End-to-End Data Pipeline</CardTitle>
                </CardHeader>
                <CardContent className="space-y-0">
                    {flowSteps.map((step, i) => (
                        <div key={step.label}>
                            <div className="flex items-start gap-4 py-3">
                                <div className={`shrink-0 w-14 text-right font-mono text-xs font-bold ${step.color}`}>
                                    {step.label}
                                </div>
                                <div className={`h-full w-px bg-zinc-700 shrink-0`} />
                                <p className="text-sm text-zinc-400 leading-relaxed">{step.text}</p>
                            </div>
                            {i < flowSteps.length - 1 && <Separator className="ml-[4.5rem] bg-zinc-800/60" />}
                        </div>
                    ))}
                </CardContent>
            </Card>

            {/* Two-column: Observability + Design */}
            <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
                <Card className="border-border/40 bg-zinc-950/40">
                    <CardHeader>
                        <CardTitle className="text-sm font-semibold text-zinc-200">Observability Hooks</CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-3">
                        {observability.map((item) => (
                            <div key={item.title} className="text-sm">
                                <span className="font-medium text-zinc-300">{item.title}:</span>{" "}
                                <span className="text-zinc-500">{item.text}</span>
                            </div>
                        ))}
                    </CardContent>
                </Card>

                <Card className="border-border/40 bg-zinc-950/40">
                    <CardHeader>
                        <CardTitle className="text-sm font-semibold text-zinc-200">Key Design Decisions</CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-3">
                        {designDecisions.map((item) => (
                            <div key={item.title} className="text-sm">
                                <span className="font-medium text-zinc-300">{item.title}:</span>{" "}
                                <span className="text-zinc-500">{item.text}</span>
                            </div>
                        ))}
                    </CardContent>
                </Card>
            </div>

            {/* Crate Map */}
            <Card className="border-border/40 bg-zinc-900/60">
                <CardHeader>
                    <CardTitle className="text-sm font-semibold text-zinc-200">Crate Dependency Map</CardTitle>
                </CardHeader>
                <CardContent>
                    <pre className="rounded-md bg-zinc-950 p-4 font-mono text-xs text-zinc-400 leading-relaxed border border-zinc-800">
                        {`context-qa/
├── Cargo.toml                  (workspace)
├── crates/
│   ├── context-core/           petgraph, serde, uuid, chrono
│   ├── context-ingest/         mock parsers (tree-sitter API)
│   ├── context-events/         in-memory store (RocksDB API)
│   ├── context-reason/         impact + intent + explanation
│   ├── context-observe/        trace + replay debugger
│   └── context-api/            axum REST (9 endpoints)
└── dashboard/                  Vite + React + shadcn/ui`}
                    </pre>
                </CardContent>
            </Card>
        </div>
    )
}
