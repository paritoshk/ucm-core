import { useState, useEffect } from "react"
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { connectLinear, getLinearStatus, importLinearIssues } from "@/lib/api"

export function IntegrationsView() {
    const [apiKey, setApiKey] = useState("")
    const [connected, setConnected] = useState(false)
    const [workspace, setWorkspace] = useState<string | undefined>(undefined)
    const [loading, setLoading] = useState(false)
    const [importLoading, setImportLoading] = useState(false)
    const [error, setError] = useState<string | null>(null)
    const [importResult, setImportResult] = useState<{ issues_count: number; events_created: number } | null>(null)

    useEffect(() => {
        getLinearStatus()
            .then((s) => {
                setConnected(s.connected)
                setWorkspace(s.workspace)
            })
            .catch(() => {
                // backend not reachable — leave disconnected
            })
    }, [])

    async function handleConnect() {
        if (!apiKey.trim()) return
        setLoading(true)
        setError(null)
        try {
            const result = await connectLinear(apiKey.trim())
            setConnected(result.connected)
            setWorkspace(result.workspace)
            setApiKey("")
        } catch (e) {
            setError(e instanceof Error ? e.message : "Connection failed")
        } finally {
            setLoading(false)
        }
    }

    async function handleDisconnect() {
        setConnected(false)
        setWorkspace(undefined)
        setImportResult(null)
    }

    async function handleImport() {
        setImportLoading(true)
        setError(null)
        try {
            const result = await importLinearIssues()
            setImportResult(result)
        } catch (e) {
            setError(e instanceof Error ? e.message : "Import failed")
        } finally {
            setImportLoading(false)
        }
    }

    return (
        <div className="space-y-6">
            <div>
                <h2 className="text-xl font-semibold text-zinc-100">Integrations</h2>
                <p className="mt-1 text-sm text-zinc-400">
                    Connect your project management tools to import requirements and features into the context graph.
                </p>
            </div>

            {/* Linear Card */}
            <Card className="border-border/60 bg-zinc-900/60">
                <CardHeader className="flex flex-row items-start justify-between pb-3">
                    <div>
                        <CardTitle className="flex items-center gap-2 text-zinc-100">
                            <svg className="h-5 w-5" viewBox="0 0 100 100" fill="none" xmlns="http://www.w3.org/2000/svg">
                                <path d="M1.22541 61.5228c-.2225-.9485.90748-1.5948 1.59636-.9068L39.0769 97.197c.6888.6888.0425 1.8183-.9068 1.5957C20.0157 94.4512 5.5490 79.9845 1.22541 61.5228ZM.00189135 46.8891c-.01764375 1.1518.92573635 2.0859 2.07759135 2.0738L46.8862 48.808c1.1519-.0122 2.0826-.9429 2.0704-2.095L48.802 2.07708c-.0122-1.15185-.9429-2.08258-2.0948-2.070771C27.6573 .225489 11.3627 9.98441 3.21894 24.5988.964374 28.6532.0196038 37.5567.00189135 46.8891ZM55.5035 1.33stagione2c-1.0875-.2357-1.7645.99208-1.0867 1.87985L97.7925 55.448c.8878.6778 2.0732.0008 1.8375-1.0867C95.5042 36.8848 79.9736 11.5478 55.5035 1.33902Z" fill="#5E6AD2"/>
                            </svg>
                            Linear
                        </CardTitle>
                        <CardDescription className="mt-1">
                            Import issues as Requirement and Feature entities in the context graph.
                        </CardDescription>
                    </div>
                    <Badge
                        variant="outline"
                        className={connected
                            ? "border-emerald-500/40 bg-emerald-500/10 text-emerald-300"
                            : "border-zinc-600/40 bg-zinc-800/40 text-zinc-400"
                        }
                    >
                        {connected ? "Connected" : "Disconnected"}
                    </Badge>
                </CardHeader>

                <CardContent className="space-y-4">
                    {!connected ? (
                        <div className="space-y-3">
                            <p className="text-xs text-zinc-500">
                                Generate an API key at{" "}
                                <span className="text-zinc-300">linear.app → Settings → API → Personal API keys</span>
                            </p>
                            <div className="flex gap-2">
                                <input
                                    type="password"
                                    placeholder="lin_api_••••••••••••••••••••••"
                                    value={apiKey}
                                    onChange={(e: React.ChangeEvent<HTMLInputElement>) => setApiKey(e.target.value)}
                                    onKeyDown={(e: React.KeyboardEvent<HTMLInputElement>) => e.key === "Enter" && handleConnect()}
                                    className="flex-1 rounded-md border border-zinc-700 bg-zinc-800/60 px-3 py-2 text-sm font-mono text-zinc-100 placeholder:text-zinc-600 focus:outline-none focus:ring-1 focus:ring-violet-500"
                                />
                                <Button
                                    onClick={handleConnect}
                                    disabled={loading || !apiKey.trim()}
                                    className="bg-violet-600 hover:bg-violet-500 text-white"
                                >
                                    {loading ? "Connecting…" : "Connect"}
                                </Button>
                            </div>
                        </div>
                    ) : (
                        <div className="space-y-4">
                            <div className="flex items-center justify-between rounded-md border border-emerald-500/20 bg-emerald-500/5 px-4 py-3">
                                <div>
                                    <p className="text-sm font-medium text-zinc-100">{workspace ?? "Linear Workspace"}</p>
                                    <p className="text-xs text-zinc-400">API key stored in session</p>
                                </div>
                                <Button
                                    variant="ghost"
                                    size="sm"
                                    onClick={handleDisconnect}
                                    className="text-zinc-400 hover:text-zinc-100"
                                >
                                    Disconnect
                                </Button>
                            </div>

                            <div className="flex items-center gap-3">
                                <Button
                                    onClick={handleImport}
                                    disabled={importLoading}
                                    className="bg-violet-600 hover:bg-violet-500 text-white"
                                >
                                    {importLoading ? "Importing…" : "Import Issues"}
                                </Button>
                                {importResult && (
                                    <p className="text-sm text-zinc-400">
                                        Imported{" "}
                                        <span className="font-medium text-emerald-400">{importResult.issues_count}</span>{" "}
                                        issues →{" "}
                                        <span className="font-medium text-violet-400">{importResult.events_created}</span>{" "}
                                        graph events
                                    </p>
                                )}
                            </div>
                        </div>
                    )}

                    {error && (
                        <p className="text-sm text-red-400 rounded-md border border-red-500/20 bg-red-500/5 px-3 py-2">
                            {error}
                        </p>
                    )}
                </CardContent>
            </Card>

            {/* GitHub placeholder */}
            <Card className="border-border/40 bg-zinc-900/30 opacity-50">
                <CardHeader className="flex flex-row items-start justify-between pb-3">
                    <div>
                        <CardTitle className="flex items-center gap-2 text-zinc-400">
                            <svg className="h-5 w-5" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M12 0C5.374 0 0 5.373 0 12c0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23A11.509 11.509 0 0 1 12 5.803c1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576C20.566 21.797 24 17.3 24 12c0-6.627-5.373-12-12-12z"/>
                            </svg>
                            GitHub
                        </CardTitle>
                        <CardDescription className="mt-1 text-zinc-600">Coming soon — import PRs and commits.</CardDescription>
                    </div>
                    <Badge variant="outline" className="border-zinc-700 bg-zinc-800/40 text-zinc-600">
                        Soon
                    </Badge>
                </CardHeader>
            </Card>
        </div>
    )
}
