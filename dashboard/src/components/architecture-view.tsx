import { useState, useEffect } from "react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { ArchitectureFlow } from "./architecture-flow"
import { ReactFlowProvider } from "@xyflow/react"
import { fetchGraphStats } from "@/lib/api"

interface GraphStats {
    entity_count: number
    edge_count: number
    avg_confidence: number
    files_tracked: number
}

export function ArchitectureView() {
    const [stats, setStats] = useState<GraphStats | null>(null)
    const [error, setError] = useState<string | null>(null)

    useEffect(() => {
        fetchGraphStats()
            .then(setStats)
            .catch(() => setError("API offline"))
    }, [])

    return (
        <div className="space-y-6">
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
                <Card className="border-border/40 bg-zinc-900/60">
                    <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                        <CardTitle className="text-sm font-medium">Total Entities</CardTitle>
                        <Badge variant="outline" className={stats ? "border-emerald-500/30 bg-emerald-500/10 text-emerald-300 text-[10px]" : "border-zinc-500/30 text-zinc-500 text-[10px]"}>
                            {stats ? "Live" : error ? "Offline" : "..."}
                        </Badge>
                    </CardHeader>
                    <CardContent>
                        <div className="text-2xl font-bold">{stats?.entity_count ?? "—"}</div>
                        <p className="text-xs text-muted-foreground">
                            Across {stats?.files_tracked ?? "—"} files
                        </p>
                    </CardContent>
                </Card>
                <Card className="border-border/40 bg-zinc-900/60">
                    <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                        <CardTitle className="text-sm font-medium">Relationships</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <div className="text-2xl font-bold">{stats?.edge_count ?? "—"}</div>
                        <p className="text-xs text-muted-foreground">Dependencies mapped</p>
                    </CardContent>
                </Card>
                <Card className="border-border/40 bg-zinc-900/60">
                    <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                        <CardTitle className="text-sm font-medium">Files Tracked</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <div className="text-2xl font-bold">{stats?.files_tracked ?? "—"}</div>
                        <p className="text-xs text-muted-foreground">Source files indexed</p>
                    </CardContent>
                </Card>
                <Card className="border-border/40 bg-zinc-900/60">
                    <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                        <CardTitle className="text-sm font-medium">Avg Confidence</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <div className="text-2xl font-bold">
                            {stats ? `${Math.round(stats.avg_confidence * 100)}%` : "—"}
                        </div>
                        <p className="text-xs text-muted-foreground">Bayesian edge score</p>
                    </CardContent>
                </Card>
            </div>

            <Card className="col-span-4 border-border/40 bg-zinc-950/40">
                <CardHeader>
                    <CardTitle>System Architecture</CardTitle>
                    <CardDescription>
                        Live visualization of the context graph from the Rust API.
                        Nodes represent services, functions, and requirements.
                        Edges represent dependencies with confidence scores.
                    </CardDescription>
                </CardHeader>
                <CardContent className="pl-2">
                    <ReactFlowProvider>
                        <ArchitectureFlow />
                    </ReactFlowProvider>
                </CardContent>
            </Card>
        </div>
    )
}
