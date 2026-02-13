import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { ArchitectureFlow } from "./architecture-flow"
import { ReactFlowProvider } from "@xyflow/react"

export function ArchitectureView() {
    return (
        <div className="space-y-6">
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
                <Card className="border-border/40 bg-zinc-900/60">
                    <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                        <CardTitle className="text-sm font-medium">Total Entities</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <div className="text-2xl font-bold">7</div>
                        <p className="text-xs text-muted-foreground">Across 5 modules</p>
                    </CardContent>
                </Card>
                <Card className="border-border/40 bg-zinc-900/60">
                    <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                        <CardTitle className="text-sm font-medium">Relationships</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <div className="text-2xl font-bold">5</div>
                        <p className="text-xs text-muted-foreground">Dependencies mapped</p>
                    </CardContent>
                </Card>
                <Card className="border-border/40 bg-zinc-900/60">
                    <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                        <CardTitle className="text-sm font-medium">Graph Depth</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <div className="text-2xl font-bold">3</div>
                        <p className="text-xs text-muted-foreground">Max chain length</p>
                    </CardContent>
                </Card>
                <Card className="border-border/40 bg-zinc-900/60">
                    <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                        <CardTitle className="text-sm font-medium">Avg Confidence</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <div className="text-2xl font-bold">85%</div>
                        <p className="text-xs text-muted-foreground">Bayesian score</p>
                    </CardContent>
                </Card>
            </div>

            <Card className="col-span-4 border-border/40 bg-zinc-950/40">
                <CardHeader>
                    <CardTitle>System Architecture</CardTitle>
                    <CardDescription>
                        Live visualization of the context graph. Nodes represent services, functions, and requirements.
                        Edges represent dependencies and data flow with confidence scores.
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
