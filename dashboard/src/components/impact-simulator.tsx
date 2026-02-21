import { useState, useEffect, useCallback } from "react"
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { ScrollArea } from "@/components/ui/scroll-area"
import {
    fetchEntities,
    fetchEdges,
    analyzeImpact,
    generateIntent,
    type ApiEntity,
    type ApiEdge,
    type ImpactReport,
    type TestIntentReport,
} from "@/lib/api"
import { Lightbulb, AlertTriangle, CheckCircle, BrainCircuit } from "lucide-react"

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
    const [intentReport, setIntentReport] = useState<TestIntentReport | null>(null)

    const [loading, setLoading] = useState(false)
    const [intentLoading, setIntentLoading] = useState(false)
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
                setError("Cannot connect to Rust API. Is it running? Start with: cargo run --bin ucm-api")
            }
        }
        loadGraph()
    }, [])

    const runSimulation = useCallback(async () => {
        if (!selectedEntity) return
        setLoading(true)
        setError(null)
        setReport(null)
        setIntentReport(null) // Clear previous intent report

        try {
            // Parse file_path and symbol from the entity ID
            const parts = selectedEntity.id.split("#")
            const symbol = parts[1] || selectedEntity.name
            const pathPart = parts[0].split("/").slice(3).join("/")
            const filePath = pathPart || selectedEntity.file_path

            const result = await analyzeImpact([{ file_path: filePath, symbol }])
            setReport(result)
        } catch (err) {
            setError(err instanceof Error ? err.message : "Impact analysis failed")
        } finally {
            setLoading(false)
        }
    }, [selectedEntity])

    const runIntentGeneration = useCallback(async () => {
        if (!selectedEntity || !report) return
        setIntentLoading(true)
        try {
            const parts = selectedEntity.id.split("#")
            const symbol = parts[1] || selectedEntity.name
            const pathPart = parts[0].split("/").slice(3).join("/")
            const filePath = pathPart || selectedEntity.file_path

            const result = await generateIntent([{ file_path: filePath, symbol }])
            setIntentReport(result)
        } catch (err) {
            setError(err instanceof Error ? err.message : "Intent generation failed")
        } finally {
            setIntentLoading(false)
        }
    }, [selectedEntity, report])

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
                            ucm-reason
                        </code>{" "}
                        engine runs reverse BFS with Bayesian confidence scoring to predict impacts and suggest test plans.
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
                                    ({entities.length} entities)
                                </span>
                            )}
                        </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-2 max-h-[400px] overflow-y-auto">
                        {entities.length === 0 && apiConnected !== false && (
                            <div className="py-8 text-center text-sm text-zinc-500">Loading entities...</div>
                        )}
                        {entities.map((entity) => (
                            <button
                                key={entity.id}
                                onClick={() => {
                                    setSelectedEntity(entity)
                                    setReport(null)
                                    setIntentReport(null)
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
                            {loading ? "Analyzing..." : "Simulate Change"}
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
                                    ({edges.length} edges)
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

            {/* Impact Report */}
            {report && (
                <Card className="border-border/40 bg-zinc-900/60 animate-in fade-in slide-in-from-bottom-4 duration-500">
                    <CardHeader>
                        <div className="flex items-center justify-between">
                            <div>
                                <CardTitle className="text-lg text-zinc-200">Impact Analysis</CardTitle>
                                <CardDescription>
                                    Bayesian impact propagation results from the Rust engine.
                                </CardDescription>
                            </div>
                            <div className="flex items-center gap-2">
                                <Button
                                    variant="outline"
                                    size="sm"
                                    onClick={runIntentGeneration}
                                    disabled={intentLoading || !!intentReport}
                                    className="border-violet-500/50 bg-violet-500/10 text-violet-300 hover:bg-violet-500/20"
                                >
                                    {intentLoading ? (
                                        "Generating..."
                                    ) : (
                                        <>
                                            <BrainCircuit className="mr-2 h-4 w-4" />
                                            Generate Test Plan
                                        </>
                                    )}
                                </Button>
                            </div>
                        </div>
                        <div className="flex gap-2 mt-4">
                            <Badge variant="outline" className="border-red-500/30 bg-red-500/10 text-red-300">
                                {report.stats.directly_impacted} direct
                            </Badge>
                            <Badge variant="outline" className="border-amber-500/30 bg-amber-500/10 text-amber-300">
                                {report.stats.indirectly_impacted} indirect
                            </Badge>
                            <Badge variant="outline" className="border-emerald-500/30 bg-emerald-500/10 text-emerald-300">
                                {report.stats.not_impacted} safe
                            </Badge>
                        </div>
                    </CardHeader>
                    <CardContent className="space-y-4">
                        {/* Direct & Indirect Impacts */}
                        {[...report.direct_impacts, ...report.indirect_impacts].map((impact, i) => {
                            const config = report.direct_impacts.includes(impact) ? levelConfig.direct : levelConfig.indirect
                            return (
                                <div key={`imp-${i}`} className={`rounded-md border px-4 py-3 ${config.className}`}>
                                    <div className="flex items-center justify-between">
                                        <div className="flex items-center gap-2">
                                            <span className={`inline-block h-2 w-2 rounded-full ${config.dotClass}`} />
                                            <span className="text-sm font-medium">{impact.name}</span>
                                        </div>
                                        <Badge variant="outline" className={config.className}>
                                            {Math.round(impact.confidence * 100)}% Confidence
                                        </Badge>
                                    </div>

                                    <div className="mt-3 pl-4 border-l-2 border-white/10 space-y-2">
                                        <p className="text-xs opacity-90 font-medium text-zinc-200">Why?</p>

                                        {/* Explanation Chain Steps */}
                                        {impact.explanation_chain && impact.explanation_chain.steps.map((step, s) => (
                                            <div key={s} className="text-xs grid grid-cols-[20px_1fr] gap-2">
                                                <span className="text-zinc-500 font-mono">{step.step}.</span>
                                                <div className="space-y-0.5">
                                                    <span className="text-zinc-300">{step.inference}</span>
                                                    <div className="text-zinc-500 italic text-[10px]">{step.evidence}</div>
                                                </div>
                                            </div>
                                        ))}

                                        {!impact.explanation_chain && (
                                            <p className="text-xs opacity-80">{impact.reason}</p>
                                        )}
                                    </div>
                                </div>
                            )
                        })}

                        {/* Logic fallback if no impacts found */}
                        {report.direct_impacts.length === 0 && report.indirect_impacts.length === 0 && (
                            <div className="text-center py-8 text-zinc-500 text-sm border border-dashed border-zinc-800 rounded-md">
                                <CheckCircle className="h-8 w-8 mx-auto mb-2 text-emerald-500/50" />
                                No impacts detected. The change appears safely isolated.
                            </div>
                        )}
                    </CardContent>
                </Card>
            )}

            {/* Test Intent Report */}
            {intentReport && (
                <Card className="border-violet-500/30 bg-violet-500/5 animate-in fade-in slide-in-from-bottom-4 duration-500 mb-20">
                    <CardHeader>
                        <CardTitle className="text-lg text-violet-200 flex items-center gap-2">
                            <Lightbulb className="h-5 w-5" />
                            Recommended Test Plan
                        </CardTitle>
                        <CardDescription>
                            AI-generated test strategy based on impact topology and confidence scores.
                        </CardDescription>
                    </CardHeader>
                    <CardContent className="space-y-6">
                        {/* High Confidence Scenarios */}
                        {intentReport.high_confidence.length > 0 && (
                            <div className="space-y-3">
                                <h4 className="text-sm font-medium text-emerald-400 flex items-center gap-2">
                                    <CheckCircle className="h-4 w-4" />
                                    Priority 1: Must Test
                                </h4>
                                {intentReport.high_confidence.map((scenario, i) => (
                                    <div key={i} className="rounded border border-emerald-500/20 bg-emerald-500/5 p-3 text-sm">
                                        <div className="font-medium text-emerald-200 mb-1">{scenario.description}</div>
                                        <div className="text-xs text-zinc-400 mb-2">Target: {scenario.related_entity} &middot; {Math.round(scenario.confidence * 100)}% confidence</div>
                                        <div className="text-xs text-zinc-500 italic border-t border-emerald-500/10 pt-2 mt-2">
                                            "{scenario.rationale}"
                                        </div>
                                    </div>
                                ))}
                            </div>
                        )}

                        {/* Risks */}
                        {intentReport.risks.length > 0 && (
                            <div className="space-y-2">
                                <h4 className="text-sm font-medium text-amber-400 flex items-center gap-2">
                                    <AlertTriangle className="h-4 w-4" />
                                    Risks
                                </h4>
                                <ul className="space-y-2">
                                    {intentReport.risks.map((risk, i) => (
                                        <li key={i} className="text-xs pl-2 border-l-2 border-amber-500/30">
                                            <div className="flex items-center gap-2 mb-0.5">
                                                <Badge variant="outline" className={
                                                    risk.severity === "High"
                                                        ? "text-[10px] border-red-500/30 text-red-300"
                                                        : risk.severity === "Medium"
                                                            ? "text-[10px] border-amber-500/30 text-amber-300"
                                                            : "text-[10px] border-zinc-500/30 text-zinc-300"
                                                }>{risk.severity}</Badge>
                                                <span className="text-amber-200/80">{risk.description}</span>
                                            </div>
                                            <div className="text-zinc-500 italic">{risk.mitigation}</div>
                                        </li>
                                    ))}
                                </ul>
                            </div>
                        )}

                        {/* Coverage Gaps */}
                        {intentReport.coverage_gaps.length > 0 && (
                            <div className="space-y-2">
                                <h4 className="text-sm font-medium text-orange-400 flex items-center gap-2">
                                    <AlertTriangle className="h-4 w-4" />
                                    Coverage Gaps
                                </h4>
                                <ul className="space-y-1">
                                    {intentReport.coverage_gaps.map((gap, i) => (
                                        <li key={i} className="text-xs text-orange-200/80 pl-2 border-l-2 border-orange-500/30">
                                            <span className="font-medium">{gap.entity}:</span> {gap.description}
                                        </li>
                                    ))}
                                </ul>
                            </div>
                        )}

                        {/* Decided Not to Test */}
                        {intentReport.decided_not_to_test && intentReport.decided_not_to_test.length > 0 && (
                            <div className="space-y-2">
                                <h4 className="text-sm font-medium text-emerald-400 flex items-center gap-2">
                                    <CheckCircle className="h-4 w-4" />
                                    Decided Not to Test
                                </h4>
                                <ul className="space-y-1">
                                    {intentReport.decided_not_to_test.map((skipped, i) => (
                                        <li key={i} className="text-xs text-emerald-200/80 pl-2 border-l-2 border-emerald-500/30">
                                            <span className="font-medium">{skipped.entity}</span>
                                            <span className="text-zinc-500"> &mdash; {skipped.reason}</span>
                                            <Badge variant="outline" className="ml-2 text-[10px] border-emerald-500/30 text-emerald-300">
                                                {Math.round(skipped.confidence_of_safety * 100)}% safe
                                            </Badge>
                                        </li>
                                    ))}
                                </ul>
                            </div>
                        )}
                    </CardContent>
                </Card>
            )}
        </div>
    )
}
