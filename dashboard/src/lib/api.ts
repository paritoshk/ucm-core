/// API client for the UCM Rust backend.
///
/// Uses VITE_API_URL env var at build time (defaults to http://localhost:3001).
/// In production, set VITE_API_URL to the Railway deployment URL.

const API_BASE = import.meta.env.VITE_API_URL || "http://localhost:3001"

export interface ApiEntity {
    id: string
    name: string
    file_path: string
    kind: string
}

export interface ApiEdge {
    from: string
    to: string
    relation: string
    confidence: number
}

export interface ImpactReport {
    changes: ChangeDescription[]
    direct_impacts: ImpactEntry[]
    indirect_impacts: ImpactEntry[]
    not_impacted: NotImpactedEntry[]
    ambiguities: AmbiguityEntry[]
    stats: ReportStats
}

interface ChangeDescription {
    entity_id: string
    name: string
    change_type: string
    file_path: string
}

interface ImpactEntry {
    entity_id: string
    name: string
    confidence: number
    tier: string
    depth: number
    path: string[]
    reason: string
    explanation_chain: ExplanationChain
}

interface NotImpactedEntry {
    entity_id: string
    name: string
    confidence: number
    reason: string
    explanation_chain: ExplanationChain
}

interface ExplanationChain {
    overall_confidence: number
    steps: ExplanationStep[]
    summary: string
}

interface ExplanationStep {
    step: number
    evidence: string
    inference: string
    confidence: number
}

interface AmbiguityEntry {
    entity_id?: string
    ambiguity_type: string
    description: string
    sources: string[]
    recommendation: string
}

interface ReportStats {
    total_entities: number
    directly_impacted: number
    indirectly_impacted: number
    not_impacted: number
    max_depth_reached: number
}

export interface TestIntentReport {
    high_confidence: TestScenario[]
    medium_confidence: TestScenario[]
    low_confidence: TestScenario[]
    risks: Risk[]
    coverage_gaps: CoverageGap[]
    decided_not_to_test: SkippedEntity[]
    summary: TestIntentSummary
}

interface TestScenario {
    description: string
    related_entity: string
    confidence: number
    rationale: string
    explanation_chain: ExplanationChain
}

interface Risk {
    severity: string
    description: string
    mitigation: string
}

interface CoverageGap {
    entity: string
    description: string
    recommendation: string
}

interface SkippedEntity {
    entity: string
    reason: string
    confidence_of_safety: number
}

interface TestIntentSummary {
    total_scenarios: number
    high_count: number
    medium_count: number
    low_count: number
    risk_count: number
}

export async function fetchHealth(): Promise<{ status: string }> {
    const res = await fetch(`${API_BASE}/health`)
    return res.json()
}

export async function fetchEntities(): Promise<ApiEntity[]> {
    const res = await fetch(`${API_BASE}/graph/entities`)
    if (!res.ok) throw new Error(`Failed to fetch entities: ${res.status}`)
    return res.json()
}

export async function fetchEdges(): Promise<ApiEdge[]> {
    const res = await fetch(`${API_BASE}/graph/edges`)
    if (!res.ok) throw new Error(`Failed to fetch edges: ${res.status}`)
    return res.json()
}

export async function fetchGraphStats(): Promise<{
    entity_count: number
    edge_count: number
    avg_confidence: number
    files_tracked: number
}> {
    const res = await fetch(`${API_BASE}/graph/stats`)
    return res.json()
}

export async function analyzeImpact(
    changedEntities: { file_path: string; symbol: string }[],
    minConfidence = 0.1,
    maxDepth = 10,
): Promise<ImpactReport> {
    const res = await fetch(`${API_BASE}/impact`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
            changed_entities: changedEntities,
            min_confidence: minConfidence,
            max_depth: maxDepth,
        }),
    })
    if (!res.ok) throw new Error(`Impact analysis failed: ${res.status}`)
    return res.json()
}

export async function generateIntent(
    changedEntities: { file_path: string; symbol: string }[],
    minConfidence = 0.1,
    maxDepth = 10,
): Promise<TestIntentReport> {
    const res = await fetch(`${API_BASE}/intent`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
            changed_entities: changedEntities,
            min_confidence: minConfidence,
            max_depth: maxDepth,
        }),
    })
    if (!res.ok) throw new Error(`Test intent generation failed: ${res.status}`)
    return res.json()
}
