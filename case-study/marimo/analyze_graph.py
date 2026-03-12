#!/usr/bin/env python3
"""Analyze UCM graph export for marimo case study.

Usage: python3 analyze_graph.py results/graph.json
"""

import json
import sys
from collections import Counter, defaultdict


def main():
    if len(sys.argv) < 2:
        print("Usage: python3 analyze_graph.py <graph.json>")
        sys.exit(1)

    with open(sys.argv[1]) as f:
        graph = json.load(f)

    entities = graph.get("entities", [])
    edges = graph.get("edges", [])

    print(f"Total entities: {len(entities)}")
    print(f"Total edges:    {len(edges)}")
    print()

    # Entity type distribution
    kind_counts = Counter()
    for e in entities:
        kind = e.get("kind", {})
        if isinstance(kind, dict):
            kind_type = list(kind.keys())[0] if kind else "Unknown"
        elif isinstance(kind, str):
            kind_type = kind
        else:
            kind_type = "Unknown"
        kind_counts[kind_type] += 1

    print("Entity types:")
    for kind, count in kind_counts.most_common():
        print(f"  {kind}: {count}")
    print()

    # Edge type distribution
    edge_type_counts = Counter()
    for e in edges:
        edge_data = e.get("edge", {})
        edge_type_counts[edge_data.get("relation_type", "Unknown")] += 1

    print("Edge types:")
    for etype, count in edge_type_counts.most_common():
        print(f"  {etype}: {count}")
    print()

    # In-degree analysis (who is imported/depended on most)
    in_degree = Counter()
    out_degree = Counter()
    for e in edges:
        target = e.get("to", "")
        source = e.get("from", "")
        in_degree[target] += 1
        out_degree[source] += 1

    print("Top 20 entities by in-degree (most depended on):")
    for entity_id, count in in_degree.most_common(20):
        # Extract readable name
        name = entity_id.split("#")[-1] if "#" in entity_id else entity_id
        file_path = entity_id.split("#")[0].split("/", 3)[-1] if "/" in entity_id else ""
        print(f"  {count:4d} ← {name} ({file_path})")
    print()

    print("Top 20 entities by out-degree (most dependencies):")
    for entity_id, count in out_degree.most_common(20):
        name = entity_id.split("#")[-1] if "#" in entity_id else entity_id
        file_path = entity_id.split("#")[0].split("/", 3)[-1] if "/" in entity_id else ""
        print(f"  {count:4d} → {name} ({file_path})")
    print()

    # Connected components (simple BFS)
    adjacency = defaultdict(set)
    all_nodes = set()
    for e in edges:
        s, t = e.get("from", ""), e.get("to", "")
        adjacency[s].add(t)
        adjacency[t].add(s)
        all_nodes.add(s)
        all_nodes.add(t)

    # Add isolated entities
    for e in entities:
        all_nodes.add(e.get("id", ""))

    visited = set()
    components = []
    for node in all_nodes:
        if node in visited:
            continue
        component = set()
        queue = [node]
        while queue:
            n = queue.pop()
            if n in visited:
                continue
            visited.add(n)
            component.add(n)
            for neighbor in adjacency.get(n, []):
                if neighbor not in visited:
                    queue.append(neighbor)
        components.append(component)

    components.sort(key=len, reverse=True)
    orphans = sum(1 for c in components if len(c) == 1)

    print(f"Connected components: {len(components)}")
    print(f"  Largest: {len(components[0]) if components else 0} nodes")
    if len(components) > 1:
        print(f"  2nd largest: {len(components[1])} nodes")
    print(f"  Orphan nodes (isolated): {orphans}")
    print()

    # File coverage
    files = set()
    for e in entities:
        fp = e.get("file_path", "")
        if fp:
            files.add(fp)

    print(f"Files with entities: {len(files)}")

    # Directory breakdown
    dir_counts = Counter()
    for fp in files:
        parts = fp.split("/")
        if len(parts) >= 2:
            dir_counts["/".join(parts[:2])] += 1
        else:
            dir_counts[fp] += 1

    print("Top directories:")
    for d, count in dir_counts.most_common(15):
        print(f"  {count:4d} files  {d}/")


if __name__ == "__main__":
    main()
