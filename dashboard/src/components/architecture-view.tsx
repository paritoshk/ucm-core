import { useState } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"

const layers = [
    {
        id: "ingest",
        label: "01 Ingestion Layer",
        color: "from-blue-500 to-blue-600",
        dotColor: "bg-blue-500",
        badgeClass: "border-blue-500/30 bg-blue-500/10 text-blue-300",
        components: [
            { name: "tree-sitter", desc: "Multi-lang AST parser (56+ langs)", tag: "Rust" },
            { name: "Git Diff Parser", desc: "Structured change extraction", tag: "Rust" },
            { name: "Ticket Adapter", desc: "Requirements → structured facts", tag: "Rust" },
            { name: "API Log Parser", desc: "Traffic → behavioral model", tag: "Rust" },
            { name: "Historical Loader", desc: "Prior snapshots → delta merge", tag: "Rust" },
        ],
    },
    {
        id: "event",
        label: "02 Event Log",
        color: "from-teal-500 to-teal-600",
        dotColor: "bg-teal-500",
        badgeClass: "border-teal-500/30 bg-teal-500/10 text-teal-300",
        components: [
            { name: "Event Store", desc: "Append-only immutable event stream", tag: "Storage" },
            { name: "Schema Registry", desc: "Versioned event types + upcasting", tag: "Storage" },
            { name: "Causation Chain", desc: "Event → parent event lineage", tag: "Storage" },
        ],
    },
    {
        id: "graph",
        label: "03 Materialized Context Graph",
        color: "from-emerald-500 to-emerald-600",
        dotColor: "bg-emerald-500",
        badgeClass: "border-emerald-500/30 bg-emerald-500/10 text-emerald-300",
        components: [
            { name: "petgraph StableGraph", desc: "Nodes: entities, edges: typed deps", tag: "Rust" },
            { name: "Fact Stack", desc: "Immutable layers with ownership tracking", tag: "Rust" },
            { name: "Confidence Scorer", desc: "Bayesian fusion + temporal decay", tag: "Rust" },
            { name: "SCIP Identity", desc: "Global unique symbol strings", tag: "Rust" },
        ],
    },
    {
        id: "compute",
        label: "04 Incremental Computation",
        color: "from-amber-500 to-amber-600",
        dotColor: "bg-amber-500",
        badgeClass: "border-amber-500/30 bg-amber-500/10 text-amber-300",
        components: [
            { name: "Impact Propagator", desc: "Reverse BFS + confidence-weighted paths", tag: "Rust" },
            { name: "Blast Radius", desc: "Direct → transitive → boundary zones", tag: "Rust" },
            { name: "Early Cutoff", desc: "Whitespace edits don't propagate", tag: "Optimization" },
        ],
    },
    {
        id: "reasoning",
        label: "05 Reasoning & Test Intent",
        color: "from-red-500 to-red-600",
        dotColor: "bg-red-500",
        badgeClass: "border-red-500/30 bg-red-500/10 text-red-300",
        components: [
            { name: "Change Analyzer", desc: "Semantic classification of changes", tag: "Core" },
            { name: "Impact Reasoner", desc: "Direct, indirect, not-impacted with why", tag: "Core" },
            { name: "Test Intent Generator", desc: "Scenarios + risks + confidence tiers", tag: "Core" },
            { name: "Ambiguity Detector", desc: "Conflicts, drift, missing data flags", tag: "Core" },
            { name: "Explanation Engine", desc: "Graph path traces → human narratives", tag: "Core" },
        ],
    },
    {
        id: "observe",
        label: "06 Observability & Debug",
        color: "from-purple-500 to-purple-600",
        dotColor: "bg-purple-500",
        badgeClass: "border-purple-500/30 bg-purple-500/10 text-purple-300",
        components: [
            { name: "Decision Trace", desc: "Every reasoning step in structured log", tag: "Debug" },
            { name: "Replay Debugger", desc: "Re-derive decisions from event log", tag: "Debug" },
            { name: "Confidence Dashboard", desc: "Edge staleness + decay heatmap", tag: "Debug" },
        ],
    },
    {
        id: "api",
        label: "07 API & Dashboard",
        color: "from-cyan-500 to-cyan-600",
        dotColor: "bg-cyan-500",
        badgeClass: "border-cyan-500/30 bg-cyan-500/10 text-cyan-300",
        components: [
            { name: "Axum REST API", desc: "9 JSON endpoints for all operations", tag: "API" },
            { name: "WebSocket Stream", desc: "Real-time graph mutation feed", tag: "API" },
            { name: "Graph Viz", desc: "Interactive dependency explorer", tag: "Frontend" },
            { name: "Impact Report View", desc: "Change → test intent → confidence", tag: "Frontend" },
        ],
    },
]

export function ArchitectureView() {
    const [expandedLayer, setExpandedLayer] = useState<string | null>("ingest")

    return (
        <div className="space-y-3">
            {layers.map((layer) => {
                const isOpen = expandedLayer === layer.id
                return (
                    <Card
                        key={layer.id}
                        className={`overflow-hidden transition-all duration-300 border-border/40 ${isOpen ? "bg-zinc-900/80 shadow-lg shadow-zinc-950/50" : "bg-zinc-950/40 hover:bg-zinc-900/40"
                            }`}
                    >
                        <button
                            className="flex w-full items-center justify-between px-6 py-4 text-left"
                            onClick={() => setExpandedLayer(isOpen ? null : layer.id)}
                        >
                            <div className="flex items-center gap-3">
                                <span className={`inline-block h-3 w-3 rounded-full ${layer.dotColor} ${isOpen ? "animate-pulse" : ""}`} />
                                <span className="text-sm font-semibold tracking-wide text-zinc-200">
                                    {layer.label}
                                </span>
                            </div>
                            <div className="flex items-center gap-3">
                                <Badge variant="outline" className={layer.badgeClass}>
                                    {layer.components.length} components
                                </Badge>
                                <span
                                    className={`text-zinc-500 transition-transform duration-200 ${isOpen ? "rotate-90" : ""
                                        }`}
                                >
                                    ›
                                </span>
                            </div>
                        </button>
                        <div
                            className={`grid transition-all duration-300 ease-in-out ${isOpen ? "grid-rows-[1fr] opacity-100" : "grid-rows-[0fr] opacity-0"
                                }`}
                        >
                            <div className="overflow-hidden">
                                <div className="grid grid-cols-1 gap-3 px-6 pb-5 sm:grid-cols-2 lg:grid-cols-3">
                                    {layer.components.map((comp, i) => (
                                        <Card
                                            key={comp.name}
                                            className="border-border/30 bg-zinc-800/50 transition-all duration-200 hover:bg-zinc-800/80 hover:shadow-md"
                                            style={{ animationDelay: `${i * 50}ms` }}
                                        >
                                            <CardHeader className="pb-2 pt-4 px-4">
                                                <div className="flex items-center justify-between">
                                                    <CardTitle className="text-sm font-medium text-zinc-200">
                                                        {comp.name}
                                                    </CardTitle>
                                                    <Badge variant="secondary" className="text-[10px] px-1.5 py-0 h-5">
                                                        {comp.tag}
                                                    </Badge>
                                                </div>
                                            </CardHeader>
                                            <CardContent className="px-4 pb-4 pt-0">
                                                <p className="text-xs text-zinc-400 leading-relaxed">{comp.desc}</p>
                                            </CardContent>
                                        </Card>
                                    ))}
                                </div>
                            </div>
                        </div>
                    </Card>
                )
            })}
        </div>
    )
}
