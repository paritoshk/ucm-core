import { useCallback } from 'react';
import {
    ReactFlow,
    Controls,
    Background,
    useNodesState,
    useEdgesState,
    addEdge,
    type Connection,
    MarkerType,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';

const initialNodes = [
    {
        id: '1',
        position: { x: 0, y: 0 },
        data: { label: 'JIRA-AUTH-42' },
        style: {
            background: '#18181b',
            color: '#e4e4e7',
            border: '1px solid #3f3f46',
            padding: '10px 20px',
            borderRadius: '8px',
            borderLeft: '4px solid #60a5fa'
        },
    },
    {
        id: '2',
        position: { x: 200, y: 0 },
        data: { label: 'validateToken()' },
        style: {
            background: '#18181b',
            color: '#e4e4e7',
            border: '1px solid #3f3f46',
            padding: '10px 20px',
            borderRadius: '8px',
            borderLeft: '4px solid #34d399'
        },
    },
    {
        id: '3',
        position: { x: 400, y: 0 },
        data: { label: 'authMiddleware()' },
        style: {
            background: '#18181b',
            color: '#e4e4e7',
            border: '1px solid #3f3f46',
            padding: '10px 20px',
            borderRadius: '8px',
            borderLeft: '4px solid #a78bfa'
        },
    },
    {
        id: '4',
        position: { x: 600, y: -50 },
        data: { label: 'POST /api/checkout' },
        style: {
            background: '#18181b',
            color: '#e4e4e7',
            border: '1px solid #3f3f46',
            padding: '10px 20px',
            borderRadius: '8px',
            borderLeft: '4px solid #fb923c'
        },
    },
    {
        id: '5',
        position: { x: 600, y: 50 },
        data: { label: 'processPayment()' },
        style: {
            background: '#18181b',
            color: '#e4e4e7',
            border: '1px solid #3f3f46',
            padding: '10px 20px',
            borderRadius: '8px',
            borderLeft: '4px solid #f472b6'
        },
    },
    {
        id: '6',
        position: { x: 600, y: 150 },
        data: { label: 'getUserProfile()' },
        style: {
            background: '#18181b',
            color: '#e4e4e7',
            border: '1px solid #3f3f46',
            padding: '10px 20px',
            borderRadius: '8px',
            borderLeft: '4px solid #22d3ee'
        },
    },
];

const initialEdges = [
    { id: 'e1-2', source: '1', target: '2', animated: true, label: 'requires', type: 'smoothstep', markerEnd: { type: MarkerType.ArrowClosed } },
    { id: 'e2-3', source: '2', target: '3', animated: true, label: 'imports', type: 'smoothstep', markerEnd: { type: MarkerType.ArrowClosed } },
    { id: 'e3-4', source: '3', target: '4', animated: true, label: 'protects', type: 'smoothstep', markerEnd: { type: MarkerType.ArrowClosed } },
    { id: 'e3-5', source: '3', target: '5', animated: true, label: 'calls', type: 'smoothstep', markerEnd: { type: MarkerType.ArrowClosed } },
    { id: 'e3-6', source: '3', target: '6', animated: true, label: 'calls', type: 'smoothstep', markerEnd: { type: MarkerType.ArrowClosed } },
];

export function ArchitectureFlow() {
    const [nodes, , onNodesChange] = useNodesState(initialNodes);
    const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);

    const onConnect = useCallback(
        (params: Connection) => setEdges((eds) => addEdge(params, eds)),
        [setEdges],
    );

    return (
        <div style={{ height: 500, width: '100%' }} className="border border-zinc-800 rounded-lg bg-zinc-950/50">
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
