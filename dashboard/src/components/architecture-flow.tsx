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

// Color palette for nodes based on file path
const colorPalette = [
    '#60a5fa', // blue
    '#34d399', // emerald
    '#a78bfa', // violet
    '#fb923c', // orange
    '#f472b6', // pink
    '#22d3ee', // cyan
    '#facc15', // yellow
    '#e879f9', // fuchsia
];

function getColorForFile(filePath: string, fileMap: Map<string, number>): string {
    if (!fileMap.has(filePath)) {
        fileMap.set(filePath, fileMap.size);
    }
    return colorPalette[fileMap.get(filePath)! % colorPalette.length];
}

function layoutNodes(entities: ApiEntity[]): Node[] {
    const fileMap = new Map<string, number>();
    const COLS = 3;
    const X_GAP = 280;
    const Y_GAP = 100;

    return entities.map((entity, i) => {
        const col = i % COLS;
        const row = Math.floor(i / COLS);
        const color = getColorForFile(entity.file_path, fileMap);

        return {
            id: entity.id,
            position: { x: col * X_GAP, y: row * Y_GAP },
            data: { label: entity.name },
            style: {
                background: '#18181b',
                color: '#e4e4e7',
                border: '1px solid #3f3f46',
                padding: '10px 20px',
                borderRadius: '8px',
                borderLeft: `4px solid ${color}`,
                fontSize: '12px',
            },
        };
    });
}

function layoutEdges(apiEdges: ApiEdge[]): Edge[] {
    return apiEdges.map((edge, i) => ({
        id: `e-${i}`,
        source: edge.from,
        target: edge.to,
        animated: true,
        label: edge.relation,
        type: 'smoothstep',
        markerEnd: { type: MarkerType.ArrowClosed },
        style: { stroke: '#52525b' },
        labelStyle: { fill: '#71717a', fontSize: 10, fontWeight: 500 },
    }));
}

export function ArchitectureFlow() {
    const [nodes, setNodes, onNodesChange] = useNodesState<Node>([]);
    const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        async function loadGraph() {
            try {
                const [ents, eds] = await Promise.all([fetchEntities(), fetchEdges()]);
                setNodes(layoutNodes(ents));
                setEdges(layoutEdges(eds));
                setError(null);
            } catch {
                setError("Cannot connect to Rust API. Start with: cargo run --bin ucm-api");
            } finally {
                setLoading(false);
            }
        }
        loadGraph();
    }, [setNodes, setEdges]);

    const onConnect = useCallback(
        (params: Connection) => setEdges((eds) => addEdge(params, eds)),
        [setEdges],
    );

    const statusMessage = useMemo(() => {
        if (loading) return "Connecting to Rust API...";
        if (error) return error;
        if (nodes.length === 0) return "Graph is empty. Ingest code via the API to populate.";
        return null;
    }, [loading, error, nodes.length]);

    return (
        <div style={{ height: 500, width: '100%' }} className="border border-zinc-800 rounded-lg bg-zinc-950/50 relative">
            {statusMessage && (
                <div className="absolute inset-0 flex items-center justify-center z-10">
                    <div className={`rounded-md border px-4 py-2 text-sm ${error ? 'border-red-500/30 bg-red-500/10 text-red-300' : 'border-zinc-700 bg-zinc-900 text-zinc-400'}`}>
                        {statusMessage}
                    </div>
                </div>
            )}
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
