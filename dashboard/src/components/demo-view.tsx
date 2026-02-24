import { useState, useEffect } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Button } from "@/components/ui/button"
import { analyzeImpact, type ImpactReport } from "@/lib/api"

const demoScenarios = [
    {
        title: "Change Impact Trace",
        desc: "Modify auth service, trace graph path to affected payment endpoint, explain why with dependency chain.",
        command: "ucm impact src/auth/service.ts validateToken --json",
        badge: "Impact",
        badgeClass: "border-red-500/30 bg-red-500/10 text-red-300",
    },
    {
        title: "Ambiguity Detection",
        desc: "Engine flags low-confidence edges and conflicting data sources. Surfaces drift between Jira requirements and live API traffic.",
        command: "ucm impact src/auth/service.ts validateToken --min-confidence 0.3",
        badge: "Audit",
        badgeClass: "border-amber-500/30 bg-amber-500/10 text-amber-300",
    },
    {
        title: "Graph Exploration",
        desc: "Scan source files, build a dependency graph, and export as JSON for further analysis or visualization.",
        command: "ucm graph ./my-project --export json -l typescript",
        badge: "Explore",
        badgeClass: "border-purple-500/30 bg-purple-500/10 text-purple-300",
    },
    {
        title: "Test Intent Generation",
        desc: "Generate prioritized test recommendations from impact analysis. Outputs must-test, should-test, and risk categories.",
        command: "ucm intent src/auth/service.ts validateToken --json",
        badge: "Test Plan",
        badgeClass: "border-teal-500/30 bg-teal-500/10 text-teal-300",
    },
]

export function DemoView() {
    const [liveOutput, setLiveOutput] = useState<ImpactReport | null>(null)
    const [loading, setLoading] = useState(false)
    const [error, setError] = useState<string | null>(null)
    const [hasLoaded, setHasLoaded] = useState(false)

    // Fetch live reasoning output from the Rust API using the demo graph's validateToken
    const fetchLiveOutput = async () => {
        setLoading(true)
        setError(null)
        try {
            const result = await analyzeImpact([
                { file_path: "src/auth/service.ts", symbol: "validateToken" },
            ])
            setLiveOutput(result)
            setHasLoaded(true)
        } catch {
            setError("Cannot connect to Rust API. Start with: cargo run --bin ucm-api")
        } finally {
            setLoading(false)
        }
    }

    // Auto-fetch on mount
    useEffect(() => {
        fetchLiveOutput()
    }, [])

    return (
        <div className="space-y-8">
            {/* Explanation */}
            <Card className="border-border/40 bg-zinc-900/60">
                <CardHeader>
                    <CardTitle className="text-lg text-zinc-200">How Reasoning Is Demonstrated</CardTitle>
                </CardHeader>
                <CardContent>
                    <p className="text-sm text-zinc-400 leading-relaxed">
                        Every output includes an{" "}
                        <code className="rounded bg-zinc-800 px-1.5 py-0.5 text-xs text-violet-300 font-mono">
                            explanation_chain
                        </code>{" "}
                        that records evidence, inference, and confidence for each step. The trace is replayable and diffable.
                    </p>
                </CardContent>
            </Card>

            {/* Scenario Cards */}
            <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
                {demoScenarios.map((demo) => (
                    <Card
                        key={demo.title}
                        className="border-border/40 bg-zinc-950/40 transition-all hover:bg-zinc-900/40 hover:shadow-lg hover:shadow-zinc-950/50"
                    >
                        <CardHeader className="pb-3">
                            <div className="flex items-center justify-between">
                                <CardTitle className="text-sm font-semibold text-zinc-200">{demo.title}</CardTitle>
                                <Badge variant="outline" className={demo.badgeClass}>
                                    {demo.badge}
                                </Badge>
                            </div>
                        </CardHeader>
                        <CardContent className="space-y-3">
                            <p className="text-xs text-zinc-400 leading-relaxed">{demo.desc}</p>
                            <div className="rounded-md bg-zinc-800/80 px-3 py-2 font-mono text-[11px] text-zinc-300 border border-zinc-700/50">
                                <span className="text-zinc-500">$ </span>
                                {demo.command}
                            </div>
                        </CardContent>
                    </Card>
                ))}
            </div>

            {/* Live Reasoning Output */}
            <Card className="border-border/40 bg-zinc-900/60">
                <CardHeader>
                    <div className="flex items-center justify-between">
                        <CardTitle className="text-lg text-zinc-200">
                            Live Reasoning Output
                            <Badge
                                variant="outline"
                                className={
                                    hasLoaded
                                        ? "ml-3 border-emerald-500/30 bg-emerald-500/10 text-emerald-300 text-[10px]"
                                        : error
                                            ? "ml-3 border-red-500/30 bg-red-500/10 text-red-300 text-[10px]"
                                            : "ml-3 border-zinc-500/30 bg-zinc-500/10 text-zinc-300 text-[10px]"
                                }
                            >
                                {hasLoaded ? "Live from Rust API" : error ? "API Offline" : "Loading..."}
                            </Badge>
                        </CardTitle>
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={fetchLiveOutput}
                            disabled={loading}
                            className="text-xs"
                        >
                            {loading ? "Fetching..." : "Refresh"}
                        </Button>
                    </div>
                </CardHeader>
                <CardContent>
                    {error && (
                        <div className="rounded-md border border-red-500/30 bg-red-500/10 px-3 py-2 text-xs text-red-300 mb-4">
                            {error}
                        </div>
                    )}
                    <ScrollArea className="h-[400px] rounded-md">
                        <pre className="rounded-md bg-zinc-950 p-4 font-mono text-xs text-zinc-300 leading-relaxed border border-zinc-800">
                            {liveOutput
                                ? JSON.stringify(liveOutput, null, 2)
                                : loading
                                    ? "Fetching live data from Rust engine..."
                                    : "No data available. Ensure the API is running."
                            }
                        </pre>
                    </ScrollArea>
                </CardContent>
            </Card>
        </div>
    )
}
