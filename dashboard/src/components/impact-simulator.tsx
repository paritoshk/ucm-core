import { useState, useEffect, useCallback } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Separator } from "@/components/ui/separator"
import {
    fetchEntities,
    fetchEdges,
    analyzeImpact,
    type ApiEntity,
    type ApiEdge,
    type ImpactReport,
} from "@/lib/api"

const levelConfig = {
    changed: {
        label: "Changed",
        className: "border-red-500/40 bg-red-500/10 text-red-300",
        dotClass: "bg-red-500",
    },
    direct: {
        label: "Direct Impact",
        className: "border-amber-500/40 bg-amber-500/10 text-amber-300",
        dotClass: "bg-amber-500",
    },
    indirect: {
        label: "Indirect Impact",
        className: "border-yellow-500/40 bg-yellow-500/10 text-yellow-300",
        dotClass: "bg-yellow-500",
    },
    not_impacted: {
        label: "Not Impacted",
        className: "border-emerald-500/40 bg-emerald-500/10 text-emerald-300",
        dotClass: "bg-emerald-500",
    },
}

export function ImpactSimulator() {
    const [entities, setEntities] = useState<ApiEntity[]>([])
    const [edges, setEdges] = useState<ApiEdge[]>([])
    const [selectedEntity, setSelectedEntity] = useState<ApiEntity | null>(null)
    const [report, setReport] = useState<ImpactReport | null>(null)
    const [loading, setLoading] = useState(false)
    const [error, setError] = useState<string | null>(null)
    const [apiConnected, setApiConnected] = useState<boolean | null>(null)

    // Fetch graph data from Rust API on mount
    useEffect(() => {
        async function loadGraph() {
            try {
                const [ents, eds] = await Promise.all([fetchEntities(), fetchEdges()])
                setEntities(ents)
                setEdges(eds)
                setApiConnected(true)
                setError(null)
            } catch {
                setApiConnected(false)
                setError("Cannot connect to Rust API. Is it running? Start with: cargo run --bin context-api")
            }
        }
        loadGraph()
    }, [])

    const runSimulation = useCallback(async () => {
        if (!selectedEntity) return
        setLoading(true)
        setError(null)

        try {
            // Parse file_path and symbol from the entity ID
            // ID format: scip:local/project/0.0.0/<file_path>#<symbol>
            const parts = selectedEntity.id.split("#")
            const symbol = parts[1] || selectedEntity.name
            const pathPart = parts[0].split("/").slice(3).join("/") // skip scip:local/project/0.0.0/
            const filePath = pathPart || selectedEntity.file_path

            const result = await analyzeImpact([{ file_path: filePath, symbol }])
            setReport(result)
        } catch (err) {
            setError(err instanceof Error ? err.message : "Impact analysis failed")
        } finally {
            setLoading(false)
        }
    }, [selectedEntity])

    return (
        <div className="space-y-6">
            {/* Status banner */}
            <Card className="border-border/40 bg-zinc-900/60">
                <CardHeader>
                    <div className="flex items-center justify-between">
                        <CardTitle className="text-lg text-zinc-200">Impact Simulator</CardTitle>
                        <Badge
                            variant="outline"
                            className={
                                apiConnected === true
                                    ? "border-emerald-500/30 bg-emerald-500/10 text-emerald-300"
                                    : apiConnected === false
                                        ? "border-red-500/30 bg-red-500/10 text-red-300"
                                        : "border-zinc-500/30 bg-zinc-500/10 text-zinc-300"
                            }
                        >
                            <span
                                className={`mr-1.5 inline-block h-2 w-2 rounded-full ${apiConnected === true
                                        ? "bg-emerald-400 animate-pulse"
                                        : apiConnected === false
                                            ? "bg-red-400"
                                            : "bg-zinc-400 animate-pulse"
                                    }`}
                            />
                            {apiConnected === true
                                ? "Rust API Connected"
                                : apiConnected === false
                                    ? "API Offline"
                                    : "Connecting..."}
                        </Badge>
                    </div>
                </CardHeader>
                <CardContent>
                    <p className="text-sm text-zinc-400 leading-relaxed">
                        Select an entity from the <strong>real Rust graph</strong>, simulate a change, and the Rust{" "}
                        <code className="rounded bg-zinc-800 px-1.5 py-0.5 text-xs text-violet-300 font-mono">
                            context-reason::impact::analyze_impact
                        </code>{" "}
                        engine runs reverse BFS with Bayesian confidence scoring.
                    </p>
                    {error && (
                        <div className="mt-3 rounded-md border border-red-500/30 bg-red-500/10 px-3 py-2 text-xs text-red-300">
                            {error}
                        </div>
                    )}
                </CardContent>
            </Card>

            {/* Entity Selector + Edges */}
            <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
                <Card className="border-border/40 bg-zinc-950/40">
                    <CardHeader>
                        <CardTitle className="text-sm font-semibold text-zinc-200">
                            Select Entity to Change
                            {entities.length > 0 && (
                                <span className="ml-2 text-xs font-normal text-zinc-500">
                                    ({entities.length} entities from Rust API)
                                </span>
                            )}
                        </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-2">
                        {entities.length === 0 && apiConnected !== false && (
                            <div className="py-8 text-center text-sm text-zinc-500">Loading entities...</div>
                        )}
                        {entities.map((entity) => (
                            <button
                                key={entity.id}
                                onClick={() => {
                                    setSelectedEntity(entity)
                                    setReport(null)
                                }}
                                className={`flex w-full items-center justify-between rounded-md border px-3 py-2.5 text-left transition-all ${selectedEntity?.id === entity.id
                                        ? "border-violet-500/50 bg-violet-500/10"
                                        : "border-zinc-800 bg-zinc-900/40 hover:border-zinc-700 hover:bg-zinc-800/50"
                                    }`}
                            >
                                <div>
                                    <div className="text-sm font-medium text-zinc-200">{entity.name}</div>
                                    <div className="text-xs text-zinc-500">{entity.file_path}</div>
                                </div>
                                <Badge variant="secondary" className="text-[10px]">
                                    {entity.kind}
                                </Badge>
                            </button>
                        ))}

                        <Button
                            className="mt-4 w-full"
                            onClick={runSimulation}
                            disabled={!selectedEntity || loading || !apiConnected}
                        >
                            {loading
                                ? "Analyzing..."
                                : selectedEntity
                                    ? `Simulate Change to ${selectedEntity.name}`
                                    : "Select an entity first"}
                        </Button>
                    </CardContent>
                </Card>

                {/* Graph Edges */}
                <Card className="border-border/40 bg-zinc-950/40">
                    <CardHeader>
                        <CardTitle className="text-sm font-semibold text-zinc-200">
                            Graph Edges
                            {edges.length > 0 && (
                                <span className="ml-2 text-xs font-normal text-zinc-500">
                                    ({edges.length} from Rust API)
                                </span>
                            )}
                        </CardTitle>
                    </CardHeader>
                    <CardContent>
                        <ScrollArea className="h-[340px]">
                            <div className="space-y-2">
                                {edges.map((edge, i) => {
                                    const fromName = edge.from.split("#")[1] || edge.from
                                    const toName = edge.to.split("#")[1] || edge.to
                                    const confPct = Math.round(edge.confidence * 100)
                                    return (
                                        <div
                                            key={i}
                                            className="rounded-md border border-zinc-800 bg-zinc-900/50 px-3 py-2 text-xs"
                                        >
                                            <div className="flex items-center justify-between">
                                                <span className="text-zinc-300">
                                                    {fromName} <span className="text-zinc-600">→</span> {toName}
                                                </span>
                                                <Badge variant="outline" className="text-[10px] h-5">
                                                    {confPct}%
                                                </Badge>
                                            </div>
                                            <div className="mt-1 text-zinc-500">{edge.relation}</div>
                                            <div className="mt-1.5 h-1 rounded-full bg-zinc-800 overflow-hidden">
                                                <div
                                                    className="h-full rounded-full bg-gradient-to-r from-violet-500 to-cyan-500 transition-all"
                                                    style={{ width: `${confPct}%` }}
                                                />
                                            </div>
                                        </div>
                                    )
                                })}
                            </div>
                        </ScrollArea>
                    </CardContent>
                </Card>
            </div>

            {/* Impact Report from Rust API */}
            {report && (
                <Card className="border-border/40 bg-zinc-900/60 animate-in fade-in slide-in-from-bottom-4 duration-500">
                    <CardHeader>
                        <div className="flex items-center justify-between">
                            <CardTitle className="text-lg text-zinc-200">
                                Impact Report
                                <span className="ml-2 text-xs font-normal text-zinc-500">
                                    (from Rust engine — {report.stats.traversal_duration_ms}ms)
                                </span>
                            </CardTitle>
                            <div className="flex gap-2">
                                <Badge variant="outline" className="border-red-500/30 bg-red-500/10 text-red-300">
                                    {report.stats.direct_count} direct
                                </Badge>
                                <Badge variant="outline" className="border-amber-500/30 bg-amber-500/10 text-amber-300">
                                    {report.stats.indirect_count} indirect
                                </Badge>
                                <Badge variant="outline" className="border-emerald-500/30 bg-emerald-500/10 text-emerald-300">
                                    {report.stats.not_impacted_count} safe
                                </Badge>
                            </div>
                        </div>
                    </CardHeader>
                    <CardContent className="space-y-3">
                        {/* Changed entities */}
                        {report.changed_entities.map((c, i) => {
                            const config = levelConfig.changed
                            return (
                                <div key={`ch-${i}`} className={`rounded-md border px-4 py-3 ${config.className}`}>
                                    <div className="flex items-center gap-2">
                                        <span className={`inline-block h-2 w-2 rounded-full ${config.dotClass}`} />
                                        <span className="text-sm font-medium">{c.name}</span>
                                        <Badge variant="outline" className={config.className}>
                                            {config.label}
                                        </Badge>
                                    </div>
                                </div>
                            )
                        })}

                        {/* Direct impacts */}
                        {report.direct_impacts.map((impact, i) => {
                            const config = levelConfig.direct
                            return (
                                <div key={`d-${i}`} className={`rounded-md border px-4 py-3 ${config.className}`}>
                                    <div className="flex items-center justify-between">
                                        <div className="flex items-center gap-2">
                                            <span className={`inline-block h-2 w-2 rounded-full ${config.dotClass}`} />
                                            <span className="text-sm font-medium">{impact.name}</span>
                                        </div>
                                        <div className="flex items-center gap-2">
                                            <span className="text-xs font-mono opacity-70">
                                                {Math.round(impact.confidence * 100)}%
                                            </span>
                                            <Badge variant="outline" className={config.className}>
                                                {config.label}
                                            </Badge>
                                        </div>
                                    </div>
                                    <p className="mt-1 text-xs opacity-80">{impact.reason}</p>
                                    {impact.path.length > 1 && (
                                        <p className="mt-1 text-xs opacity-60 font-mono">
                                            Path: {impact.path.map((p) => p.split("#")[1] || p).join(" → ")}
                                        </p>
                                    )}
                                </div>
                            )
                        })}

                        {/* Indirect impacts */}
                        {report.indirect_impacts.map((impact, i) => {
                            const config = levelConfig.indirect
                            return (
                                <div key={`i-${i}`} className={`rounded-md border px-4 py-3 ${config.className}`}>
                                    <div className="flex items-center justify-between">
                                        <div className="flex items-center gap-2">
                                            <span className={`inline-block h-2 w-2 rounded-full ${config.dotClass}`} />
                                            <span className="text-sm font-medium">{impact.name}</span>
                                        </div>
                                        <div className="flex items-center gap-2">
                                            <span className="text-xs font-mono opacity-70">
                                                {Math.round(impact.confidence * 100)}%
                                            </span>
                                            <Badge variant="outline" className={config.className}>
                                                {config.label}
                                            </Badge>
                                        </div>
                                    </div>
                                    <p className="mt-1 text-xs opacity-80">{impact.reason}</p>
                                    {impact.path.length > 1 && (
                                        <p className="mt-1 text-xs opacity-60 font-mono">
                                            Path: {impact.path.map((p) => p.split("#")[1] || p).join(" → ")}
                                        </p>
                                    )}
                                </div>
                            )
                        })}

                        {/* Not impacted */}
                        {report.not_impacted.map((ni, i) => {
                            const config = levelConfig.not_impacted
                            return (
                                <div key={`n-${i}`} className={`rounded-md border px-4 py-3 ${config.className}`}>
                                    <div className="flex items-center justify-between">
                                        <div className="flex items-center gap-2">
                                            <span className={`inline-block h-2 w-2 rounded-full ${config.dotClass}`} />
                                            <span className="text-sm font-medium">{ni.name}</span>
                                        </div>
                                        <div className="flex items-center gap-2">
                                            <span className="text-xs font-mono opacity-70">
                                                {Math.round(ni.confidence * 100)}%
                                            </span>
                                            <Badge variant="outline" className={config.className}>
                                                {config.label}
                                            </Badge>
                                        </div>
                                    </div>
                                    <p className="mt-1 text-xs opacity-80">{ni.reason}</p>
                                </div>
                            )
                        })}

                        <Separator className="my-2" />

                        {/* Ambiguities */}
                        {report.ambiguities.length > 0 && (
                            <div className="space-y-2">
                                <div className="text-xs font-medium text-zinc-400">Ambiguities Detected</div>
                                {report.ambiguities.map((a, i) => (
                                    <div
                                        key={i}
                                        className="rounded-md border border-amber-500/30 bg-amber-500/5 px-3 py-2 text-xs text-amber-300"
                                    >
                                        <div className="font-medium">{a.kind}</div>
                                        <p className="mt-0.5 opacity-80">{a.description}</p>
                                        <p className="mt-0.5 text-amber-400/60">{a.recommendation}</p>
                                    </div>
                                ))}
                            </div>
                        )}
                    </CardContent>
                </Card>
            )}
        </div>
    )
}
