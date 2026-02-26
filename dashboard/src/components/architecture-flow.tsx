import { useState, useEffect, useCallback, useMemo } from 'react';
import {
    ReactFlow,
    Controls,
    Background,
    useNodesState,
    useEdgesState,
    addEdge,
    type Connection,
    type Node,
    type Edge,
    MarkerType,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import { fetchEntities, fetchEdges, type ApiEntity, type ApiEdge } from '@/lib/api';

// ── Demo graph (always visible as baseline) ──────────────────────────

const demoNodes: Node[] = [
    {
        id: '1',
        position: { x: 0, y: 0 },
        data: { label: 'JIRA-AUTH-42' },
        style: {
            background: '#18181b', color: '#e4e4e7',
            border: '1px solid #3f3f46', padding: '10px 20px',
            borderRadius: '8px', borderLeft: '4px solid #60a5fa',
        },
    },
    {
        id: '2',
        position: { x: 200, y: 0 },
        data: { label: 'validateToken()' },
        style: {
            background: '#18181b', color: '#e4e4e7',
            border: '1px solid #3f3f46', padding: '10px 20px',
            borderRadius: '8px', borderLeft: '4px solid #34d399',
        },
    },
    {
        id: '3',
        position: { x: 400, y: 0 },
        data: { label: 'authMiddleware()' },
        style: {
            background: '#18181b', color: '#e4e4e7',
            border: '1px solid #3f3f46', padding: '10px 20px',
            borderRadius: '8px', borderLeft: '4px solid #a78bfa',
        },
    },
    {
        id: '4',
        position: { x: 600, y: -50 },
        data: { label: 'POST /api/checkout' },
        style: {
            background: '#18181b', color: '#e4e4e7',
            border: '1px solid #3f3f46', padding: '10px 20px',
            borderRadius: '8px', borderLeft: '4px solid #fb923c',
        },
    },
    {
        id: '5',
        position: { x: 600, y: 50 },
        data: { label: 'processPayment()' },
        style: {
            background: '#18181b', color: '#e4e4e7',
            border: '1px solid #3f3f46', padding: '10px 20px',
            borderRadius: '8px', borderLeft: '4px solid #f472b6',
        },
    },
    {
        id: '6',
        position: { x: 600, y: 150 },
        data: { label: 'getUserProfile()' },
        style: {
            background: '#18181b', color: '#e4e4e7',
            border: '1px solid #3f3f46', padding: '10px 20px',
            borderRadius: '8px', borderLeft: '4px solid #22d3ee',
        },
    },
];

const demoEdges: Edge[] = [
    { id: 'e1-2', source: '1', target: '2', animated: true, label: 'requires', type: 'smoothstep', markerEnd: { type: MarkerType.ArrowClosed }, style: { stroke: '#52525b' }, labelStyle: { fill: '#71717a', fontSize: 10, fontWeight: 500 } },
    { id: 'e2-3', source: '2', target: '3', animated: true, label: 'imports', type: 'smoothstep', markerEnd: { type: MarkerType.ArrowClosed }, style: { stroke: '#52525b' }, labelStyle: { fill: '#71717a', fontSize: 10, fontWeight: 500 } },
    { id: 'e3-4', source: '3', target: '4', animated: true, label: 'protects', type: 'smoothstep', markerEnd: { type: MarkerType.ArrowClosed }, style: { stroke: '#52525b' }, labelStyle: { fill: '#71717a', fontSize: 10, fontWeight: 500 } },
    { id: 'e3-5', source: '3', target: '5', animated: true, label: 'calls', type: 'smoothstep', markerEnd: { type: MarkerType.ArrowClosed }, style: { stroke: '#52525b' }, labelStyle: { fill: '#71717a', fontSize: 10, fontWeight: 500 } },
    { id: 'e3-6', source: '3', target: '6', animated: true, label: 'calls', type: 'smoothstep', markerEnd: { type: MarkerType.ArrowClosed }, style: { stroke: '#52525b' }, labelStyle: { fill: '#71717a', fontSize: 10, fontWeight: 500 } },
];

// ── Live data helpers ────────────────────────────────────────────────

const colorPalette = ['#60a5fa', '#34d399', '#a78bfa', '#fb923c', '#f472b6', '#22d3ee', '#facc15', '#e879f9'];

function getColorForFile(filePath: string, fileMap: Map<string, number>): string {
    if (!fileMap.has(filePath)) fileMap.set(filePath, fileMap.size);
    return colorPalette[fileMap.get(filePath)! % colorPalette.length];
}

// React Flow uses node IDs in DOM element IDs and CSS selectors.
// SCIP IDs contain ':', '/', '.', '#' which break querySelector.
// Replace every non-alphanumeric character with '_'.
function safeId(scip: string): string {
    return scip.replace(/[^a-zA-Z0-9_-]/g, '_');
}

function layoutNodes(entities: ApiEntity[], apiEdges: ApiEdge[]): Node[] {
    const fileMap = new Map<string, number>();
    const entityIds = new Set(entities.map(e => e.id));

    // Validate edges: only use edges whose from/to match existing entity IDs
    const validEdges = apiEdges.filter(edge => {
        const fromOk = entityIds.has(edge.from);
        const toOk = entityIds.has(edge.to);
        if (!fromOk || !toOk) {
            console.warn('[UCM] Edge references unknown entity:', {
                from: edge.from, fromExists: fromOk,
                to: edge.to, toExists: toOk,
            });
        }
        return fromOk && toOk;
    });

    console.log(`[UCM] Layout: ${entities.length} entities, ${validEdges.length}/${apiEdges.length} valid edges`);

    // Build adjacency to compute a rough "depth" per node (BFS from roots).
    // Roots = nodes with no incoming edges.
    const inDegree = new Map<string, number>(entities.map(e => [e.id, 0]));
    for (const edge of validEdges) {
        inDegree.set(edge.to, (inDegree.get(edge.to) ?? 0) + 1);
    }
    const outEdges = new Map<string, string[]>(entities.map(e => [e.id, []]));
    for (const edge of validEdges) {
        outEdges.get(edge.from)?.push(edge.to);
    }

    const depth = new Map<string, number>(entities.map(e => [e.id, 0]));
    const queue = entities.filter(e => (inDegree.get(e.id) ?? 0) === 0).map(e => e.id);
    let visited = new Set(queue);
    while (queue.length > 0) {
        const id = queue.shift()!;
        const d = depth.get(id) ?? 0;
        for (const neighbor of outEdges.get(id) ?? []) {
            if (!visited.has(neighbor)) {
                visited.add(neighbor);
                depth.set(neighbor, d + 1);
                queue.push(neighbor);
            }
        }
    }

    // Group by depth column, spread rows within each column.
    const cols = new Map<number, string[]>();
    for (const entity of entities) {
        const d = depth.get(entity.id) ?? 0;
        if (!cols.has(d)) cols.set(d, []);
        cols.get(d)!.push(entity.id);
    }

    // If ALL nodes ended up at depth 0 (single column), redistribute into a grid
    // so that edges are visually distinguishable.
    const maxDepth = Math.max(...Array.from(depth.values()));
    if (maxDepth === 0 && entities.length > 1 && validEdges.length > 0) {
        console.warn('[UCM] All nodes at depth 0 despite edges — using grid fallback layout');
        // Build a simple 2-column layout: sources on left, targets on right
        const hasOutgoing = new Set(validEdges.map(e => e.from));
        const hasIncoming = new Set(validEdges.map(e => e.to));
        const leftCol: string[] = [];
        const rightCol: string[] = [];
        for (const entity of entities) {
            if (hasOutgoing.has(entity.id) && !hasIncoming.has(entity.id)) {
                leftCol.push(entity.id);
            } else if (hasIncoming.has(entity.id) && !hasOutgoing.has(entity.id)) {
                rightCol.push(entity.id);
            } else {
                // Both or neither — put on left
                leftCol.push(entity.id);
            }
        }
        cols.clear();
        cols.set(0, leftCol);
        if (rightCol.length > 0) cols.set(1, rightCol);
    }

    const positions = new Map<string, { x: number; y: number }>();
    const X_GAP = 280;
    const Y_GAP = 110;
    for (const [col, ids] of cols) {
        ids.forEach((id, row) => {
            positions.set(id, { x: col * X_GAP, y: row * Y_GAP });
        });
    }

    return entities.map(entity => {
        const color = getColorForFile(entity.file_path, fileMap);
        return {
            id: safeId(entity.id),
            position: positions.get(entity.id) ?? { x: 0, y: 0 },
            data: { label: entity.name },
            style: {
                background: '#18181b', color: '#e4e4e7',
                border: '1px solid #3f3f46', padding: '10px 20px',
                borderRadius: '8px', borderLeft: `4px solid ${color}`,
                fontSize: '12px',
            },
        };
    });
}

function layoutEdges(apiEdges: ApiEdge[]): Edge[] {
    return apiEdges.map((edge, i) => ({
        id: `e-${i}`,
        source: safeId(edge.from),
        target: safeId(edge.to),
        animated: true,
        label: edge.relation,
        type: 'smoothstep',
        markerEnd: { type: MarkerType.ArrowClosed },
        style: { stroke: '#52525b' },
        labelStyle: { fill: '#71717a', fontSize: 10, fontWeight: 500 },
    }));
}

// ── Component ────────────────────────────────────────────────────────

export function ArchitectureFlow() {
    const [nodes, setNodes, onNodesChange] = useNodesState<Node>(demoNodes);
    const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>(demoEdges);
    const [source, setSource] = useState<'demo' | 'live'>('demo');

    useEffect(() => {
        async function tryLoadLive() {
            try {
                const [ents, eds] = await Promise.all([fetchEntities(), fetchEdges()]);
                console.log(`[UCM] API response: ${ents.length} entities, ${eds.length} edges`);
                if (ents.length > 0) {
                    console.log('[UCM] Sample entity id:', ents[0].id);
                }
                if (eds.length > 0) {
                    console.log('[UCM] Sample edge from/to:', eds[0].from, '->', eds[0].to);
                }
                // Only swap to live data if the API actually has entities AND edges
                if (ents.length > 0 && eds.length > 0) {
                    const liveNodes = layoutNodes(ents, eds);
                    const liveEdges = layoutEdges(eds);
                    // Validate: check all edge source/target match a node id
                    const nodeIds = new Set(liveNodes.map(n => n.id));
                    const unmatchedEdges = liveEdges.filter(
                        e => !nodeIds.has(e.source) || !nodeIds.has(e.target)
                    );
                    if (unmatchedEdges.length > 0) {
                        console.warn('[UCM] Unmatched edges after safeId:', unmatchedEdges.map(e => `${e.source} -> ${e.target}`));
                        console.warn('[UCM] Available node IDs:', Array.from(nodeIds));
                    }
                    console.log(`[UCM] Rendering: ${liveNodes.length} nodes, ${liveEdges.length} edges (${unmatchedEdges.length} unmatched)`);
                    setNodes(liveNodes);
                    setEdges(liveEdges);
                    setSource('live');
                } else {
                    console.log('[UCM] Insufficient live data, keeping demo graph');
                }
            } catch (err) {
                console.warn('[UCM] API fetch failed, keeping demo graph:', err);
            }
        }
        tryLoadLive();
    }, [setNodes, setEdges]);

    const onConnect = useCallback(
        (params: Connection) => setEdges((eds) => addEdge(params, eds)),
        [setEdges],
    );

    const badge = useMemo(() => {
        if (source === 'live') return { text: 'Live from API', cls: 'border-emerald-500/30 bg-emerald-500/10 text-emerald-300' };
        return { text: 'Demo Graph', cls: 'border-amber-500/30 bg-amber-500/10 text-amber-300' };
    }, [source]);

    return (
        <div style={{ height: 500, width: '100%' }} className="border border-zinc-800 rounded-lg bg-zinc-950/50 relative">
            <div className="absolute top-2 right-2 z-10">
                <span className={`rounded-full border px-2 py-0.5 text-[10px] font-medium ${badge.cls}`}>
                    {badge.text}
                </span>
            </div>
            <ReactFlow
                nodes={nodes}
                edges={edges}
                onNodesChange={onNodesChange}
                onEdgesChange={onEdgesChange}
                onConnect={onConnect}
                fitView
                colorMode="dark"
            >
                <Background />
                <Controls />
            </ReactFlow>
        </div>
    );
}
