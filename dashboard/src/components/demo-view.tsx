import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { ScrollArea } from "@/components/ui/scroll-area"

const demoScenarios = [
    {
        title: "Change Impact Trace",
        desc: "Modify auth service, trace graph path to affected payment endpoint, explain why with dependency chain.",
        command: "contextqa impact --change src/auth/service.ts --output json --explain",
        badge: "Impact",
        badgeClass: "border-red-500/30 bg-red-500/10 text-red-300",
    },
    {
        title: "Ambiguity Detection",
        desc: "Jira says reset via email but API logs show SMS — system flags conflict and cites evidence.",
        command: "contextqa audit --source jira,api-logs --detect conflicts",
        badge: "Audit",
        badgeClass: "border-amber-500/30 bg-amber-500/10 text-amber-300",
    },
    {
        title: "Reasoning Replay",
        desc: "Re-derive a past decision from the event log. Show full reasoning chain with diffs.",
        command: "contextqa replay --event-id <uuid> --verbose --diff",
        badge: "Debug",
        badgeClass: "border-purple-500/30 bg-purple-500/10 text-purple-300",
    },
    {
        title: "Confidence Decay",
        desc: "Query stale dependencies. Show which edges fell below threshold with re-verification guidance.",
        command: "contextqa health --threshold 0.6 --show-decay",
        badge: "Health",
        badgeClass: "border-teal-500/30 bg-teal-500/10 text-teal-300",
    },
]

const sampleReasoning = {
    change: "src/auth/service.ts#validateToken()",
    impact_report: {
        direct: [
            {
                entity: "src/api/middleware.ts#authMiddleware",
                confidence: 0.95,
                reason: "Imports validateToken directly",
            },
        ],
        indirect: [
            {
                entity: "src/payments/checkout.ts#processPayment",
                confidence: 0.72,
                reason: "authMiddleware → routeHandler → processPayment (3-hop, confidence decays)",
                path: ["authMiddleware", "protectedRoutes", "checkoutRoute", "processPayment"],
            },
        ],
        not_impacted: [
            {
                entity: "src/admin/reports.ts",
                confidence: 0.88,
                reason: "No graph path exists; uses separate admin auth flow",
            },
        ],
        ambiguities: [
            {
                type: "drift",
                detail: "Jira AUTH-42 says OAuth2 only, but API logs show JWT bearer tokens",
                sources: { jira: "OAuth2", api_logs: "JWT Bearer" },
                recommendation: "Verify with team, test both flows",
            },
        ],
    },
    explanation_chain: [
        {
            step: 1,
            evidence: "git diff shows validateToken() signature changed",
            inference: "Return type changed from boolean to Result<Claims, AuthError>",
            confidence: 1.0,
        },
        {
            step: 2,
            evidence: "Static analysis found 3 call sites via reverse BFS",
            inference: "All callers must handle the new Result type",
            confidence: 0.95,
        },
        {
            step: 3,
            evidence: "API logs show /checkout called 1.2M times/day",
            inference: "Payment flow is a high-traffic indirect dependency",
            confidence: 0.72,
        },
    ],
}

export function DemoView() {
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

            {/* Sample JSON Output */}
            <Card className="border-border/40 bg-zinc-900/60">
                <CardHeader>
                    <CardTitle className="text-lg text-zinc-200">Sample Reasoning Output</CardTitle>
                </CardHeader>
                <CardContent>
                    <ScrollArea className="h-[400px] rounded-md">
                        <pre className="rounded-md bg-zinc-950 p-4 font-mono text-xs text-zinc-300 leading-relaxed border border-zinc-800">
                            {JSON.stringify(sampleReasoning, null, 2)}
                        </pre>
                    </ScrollArea>
                </CardContent>
            </Card>
        </div>
    )
}
