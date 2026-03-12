#!/usr/bin/env python3
"""
Analyze UCM graph export to find high-impact, low-test-coverage entities
that are prime targets for contribution.
"""

import json
import sys
from collections import defaultdict, Counter
from pathlib import Path

GRAPH_PATH = Path(__file__).parent / "results" / "graph.json"
OUTPUT_PATH = Path(__file__).parent / "results" / "contribution-analysis.txt"


def is_test_file(file_path: str) -> bool:
    """Check if a file path is in a test directory."""
    parts = file_path.split("/")
    for part in parts:
        if part in ("_tests", "tests", "test", "_test"):
            return True
    if file_path.endswith("_test.py") or file_path.endswith("_tests.py"):
        return True
    return False


def get_entity_kind(entity: dict) -> str:
    """Extract the kind string from an entity."""
    return list(entity["kind"].keys())[0]


def main():
    print("Loading graph.json...")
    with open(GRAPH_PATH) as f:
        graph = json.load(f)

    entities = graph["entities"]
    edges = graph["edges"]

    print(f"  Entities: {len(entities)}")
    print(f"  Edges: {len(edges)}")

    # Build lookup maps
    entity_by_id = {e["id"]: e for e in entities}

    # Compute in-degree (how many entities depend on this entity)
    # An edge "from -> to" with relation "DependsOn" means `from` depends on `to`
    # So in-degree for `to` = count of edges pointing TO it
    in_degree = Counter()
    out_degree = Counter()
    dependents = defaultdict(set)  # entity -> set of entities that depend on it
    dependencies = defaultdict(set)  # entity -> set of entities it depends on

    for edge in edges:
        src = edge["from"]
        dst = edge["to"]
        rel = edge["edge"]["relation_type"]

        if rel == "DependsOn":
            # src depends on dst => dst has higher in-degree
            in_degree[dst] += 1
            out_degree[src] += 1
            dependents[dst].add(src)
            dependencies[src].add(dst)

    # Identify test entities and their imports
    test_entity_ids = set()
    non_test_entity_ids = set()
    for e in entities:
        fp = e.get("file_path", "")
        if is_test_file(fp):
            test_entity_ids.add(e["id"])
        else:
            non_test_entity_ids.add(e["id"])

    # Find entities imported by test files
    # If a test entity depends on X, then X is "tested" (imported by tests)
    tested_entity_ids = set()
    for tid in test_entity_ids:
        for dep_id in dependencies.get(tid, set()):
            tested_entity_ids.add(dep_id)
        # Also: if test entities are in dependents of X, X is tested
    for eid in list(dependents.keys()):
        for dep_id in dependents[eid]:
            if dep_id in test_entity_ids:
                tested_entity_ids.add(eid)

    # Collect file-level info: map file_path -> set of entity ids
    file_entities = defaultdict(set)
    for e in entities:
        fp = e.get("file_path", "")
        if fp:
            file_entities[fp].add(e["id"])

    # Also check if any test file imports anything from the same module
    # by looking at file-level coverage
    tested_files = set()
    for eid in tested_entity_ids:
        e = entity_by_id.get(eid)
        if e:
            tested_files.add(e.get("file_path", ""))

    lines = []
    lines.append("=" * 80)
    lines.append("UCM GRAPH CONTRIBUTION ANALYSIS")
    lines.append("=" * 80)
    lines.append(f"\nGraph stats: {len(entities)} entities, {len(edges)} edges")
    lines.append(f"Test entities: {len(test_entity_ids)}")
    lines.append(f"Non-test entities: {len(non_test_entity_ids)}")
    lines.append(f"Entities imported by tests: {len(tested_entity_ids)}")

    # =========================================================================
    # 1. Top 20 entities by in-degree WITHOUT test coverage
    # =========================================================================
    lines.append("\n" + "=" * 80)
    lines.append("1. TOP 20 HIGH IN-DEGREE ENTITIES WITHOUT TEST IMPORTS")
    lines.append("   (High-impact code that no test file imports)")
    lines.append("=" * 80)

    # Filter: non-test entities, not in tested_entity_ids
    untested_high_indegree = []
    for eid, deg in in_degree.most_common():
        if eid in test_entity_ids:
            continue
        if eid in tested_entity_ids:
            continue
        e = entity_by_id.get(eid)
        if not e:
            continue
        untested_high_indegree.append((eid, deg, e))
        if len(untested_high_indegree) >= 20:
            break

    for rank, (eid, deg, e) in enumerate(untested_high_indegree, 1):
        kind = get_entity_kind(e)
        fp = e.get("file_path", "?")
        name = e.get("name", "?")
        num_dependents = len(dependents.get(eid, set()))
        lines.append(f"\n  #{rank:2d}  in-degree={deg}, dependents={num_dependents}")
        lines.append(f"       {kind}: {name}")
        lines.append(f"       File: {fp}")
        # Show a few dependents
        dep_files = set()
        for did in list(dependents.get(eid, set()))[:10]:
            de = entity_by_id.get(did)
            if de:
                dep_files.add(de.get("file_path", "?"))
        if dep_files:
            lines.append(f"       Used by files: {', '.join(sorted(dep_files)[:5])}")

    # =========================================================================
    # 2. Hidden dependency chains spanning 3+ modules
    # =========================================================================
    lines.append("\n" + "=" * 80)
    lines.append("2. HIDDEN DEPENDENCY CHAINS (3+ MODULES DEEP)")
    lines.append("   (Changes here ripple through multiple modules)")
    lines.append("=" * 80)

    def get_file(eid):
        e = entity_by_id.get(eid)
        return e.get("file_path", "") if e else ""

    def get_module(file_path):
        """Extract top-level module from file path."""
        parts = file_path.split("/")
        if len(parts) >= 1:
            return parts[0]
        return file_path

    # BFS from high in-degree non-test entities to find long chains
    # across different modules (file-level grouping)
    def find_chains_from(start_eid, max_depth=6):
        """Find dependency chains originating from start_eid via dependents.
        Returns chains that span 3+ distinct modules."""
        chains = []
        # BFS tracking (entity, path_of_files)
        queue = [(start_eid, [get_file(start_eid)])]
        visited = {start_eid}

        while queue:
            current, path = queue.pop(0)
            current_file = get_file(current)

            if len(path) >= 3:
                modules = set(get_module(f) for f in path if f)
                if len(modules) >= 3:
                    chains.append(path[:])

            if len(path) >= max_depth:
                continue

            for dep_id in dependents.get(current, set()):
                if dep_id in visited:
                    continue
                dep_file = get_file(dep_id)
                if dep_file and dep_file != current_file:
                    visited.add(dep_id)
                    queue.append((dep_id, path + [dep_file]))

        return chains

    # Pick top entities by in-degree (non-test) and find chains
    chain_results = []
    seen_chain_starts = set()
    candidates = sorted(
        [(eid, deg) for eid, deg in in_degree.items()
         if eid not in test_entity_ids],
        key=lambda x: -x[1]
    )[:50]

    for eid, deg in candidates:
        fp = get_file(eid)
        if fp in seen_chain_starts:
            continue
        seen_chain_starts.add(fp)
        chains = find_chains_from(eid, max_depth=5)
        if chains:
            # Pick the longest chain
            longest = max(chains, key=lambda c: len(set(get_module(f) for f in c)))
            modules = list(dict.fromkeys(get_module(f) for f in longest if f))
            if len(modules) >= 3:
                chain_results.append((eid, deg, longest, modules))

    # Sort by number of modules spanned
    chain_results.sort(key=lambda x: -len(x[3]))

    for i, (eid, deg, chain, modules) in enumerate(chain_results[:15], 1):
        e = entity_by_id.get(eid, {})
        name = e.get("name", "?")
        kind = get_entity_kind(e) if e.get("kind") else "?"
        lines.append(f"\n  Chain #{i}: {kind} '{name}' (in-degree={deg})")
        lines.append(f"    Modules spanned: {len(modules)}")
        lines.append(f"    Module path: {' -> '.join(modules)}")
        lines.append(f"    File chain:")
        for fp in dict.fromkeys(chain):  # deduplicate preserving order
            lines.append(f"      - {fp}")

    # =========================================================================
    # 3. "Utility" modules with surprisingly high blast radius
    # =========================================================================
    lines.append("\n" + "=" * 80)
    lines.append("3. UTILITY MODULES WITH HIGH BLAST RADIUS")
    lines.append("   (Files in _utils/, utils/, helpers/ etc. with large impact)")
    lines.append("=" * 80)

    def is_utility_file(fp):
        parts = fp.lower().split("/")
        util_markers = ("_utils", "utils", "helpers", "helper", "common", "shared", "lib")
        return any(p in util_markers for p in parts)

    # Compute file-level blast radius: for each file, how many OTHER files
    # transitively depend on entities in this file?
    file_in_degree = defaultdict(int)  # file -> total in-degree of its entities
    file_dependents_files = defaultdict(set)  # file -> set of files that directly depend on it

    for eid, deps in dependents.items():
        e = entity_by_id.get(eid)
        if not e:
            continue
        fp = e.get("file_path", "")
        if not fp:
            continue
        for dep_id in deps:
            dep_e = entity_by_id.get(dep_id)
            if dep_e:
                dep_fp = dep_e.get("file_path", "")
                if dep_fp and dep_fp != fp:
                    file_dependents_files[fp].add(dep_fp)
        file_in_degree[fp] += in_degree.get(eid, 0)

    # Compute transitive blast radius for utility files via BFS on file-level graph
    # file_dep_graph: file -> set of files that depend on it (reverse dependency)
    file_dep_graph = file_dependents_files  # already computed above

    def transitive_blast_radius(start_file, max_depth=5):
        """BFS to find all files transitively affected."""
        visited = {start_file}
        queue = [start_file]
        while queue:
            current = queue.pop(0)
            for dep_file in file_dep_graph.get(current, set()):
                if dep_file not in visited:
                    visited.add(dep_file)
                    queue.append(dep_file)
        visited.discard(start_file)
        return visited

    utility_blast = []
    all_files = set(e.get("file_path", "") for e in entities if e.get("file_path"))

    for fp in all_files:
        if not is_utility_file(fp):
            continue
        if is_test_file(fp):
            continue
        direct = file_dependents_files.get(fp, set())
        transitive = transitive_blast_radius(fp)
        entity_count = len(file_entities.get(fp, set()))
        # Check test coverage for this file
        has_test = fp in tested_files
        utility_blast.append({
            "file": fp,
            "direct_dependents": len(direct),
            "transitive_blast": len(transitive),
            "entity_count": entity_count,
            "file_in_degree": file_in_degree.get(fp, 0),
            "has_test_coverage": has_test,
        })

    # Sort by transitive blast radius
    utility_blast.sort(key=lambda x: -x["transitive_blast"])

    for i, ub in enumerate(utility_blast[:20], 1):
        test_status = "YES" if ub["has_test_coverage"] else "NO"
        lines.append(f"\n  #{i:2d}  {ub['file']}")
        lines.append(f"       Entities: {ub['entity_count']}, "
                     f"File in-degree: {ub['file_in_degree']}")
        lines.append(f"       Direct dependents: {ub['direct_dependents']} files")
        lines.append(f"       Transitive blast radius: {ub['transitive_blast']} files")
        lines.append(f"       Test coverage: {test_status}")

    # =========================================================================
    # Summary: Top contribution targets
    # =========================================================================
    lines.append("\n" + "=" * 80)
    lines.append("SUMMARY: TOP CONTRIBUTION TARGETS")
    lines.append("(High impact + low test coverage = greatest value for contributors)")
    lines.append("=" * 80)

    # Combine: untested high in-degree entities + untested utility modules
    # Group by file for actionability
    target_files = defaultdict(lambda: {
        "max_in_degree": 0, "untested_entities": 0, "total_entities": 0,
        "is_utility": False, "blast_radius": 0, "entity_names": []
    })

    for eid, deg, e in untested_high_indegree:
        fp = e.get("file_path", "")
        if not fp:
            continue
        target_files[fp]["max_in_degree"] = max(target_files[fp]["max_in_degree"], deg)
        target_files[fp]["untested_entities"] += 1
        target_files[fp]["entity_names"].append(e.get("name", "?"))

    for ub in utility_blast:
        if not ub["has_test_coverage"]:
            fp = ub["file"]
            target_files[fp]["is_utility"] = True
            target_files[fp]["blast_radius"] = ub["transitive_blast"]

    # Score: weighted combination
    scored = []
    for fp, info in target_files.items():
        score = (
            info["max_in_degree"] * 2
            + info["untested_entities"] * 5
            + info["blast_radius"]
            + (10 if info["is_utility"] else 0)
        )
        scored.append((fp, score, info))

    scored.sort(key=lambda x: -x[1])

    for i, (fp, score, info) in enumerate(scored[:15], 1):
        lines.append(f"\n  #{i:2d}  {fp}  (score={score})")
        if info["entity_names"]:
            names = ", ".join(info["entity_names"][:5])
            lines.append(f"       Key untested entities: {names}")
        lines.append(f"       Max in-degree: {info['max_in_degree']}, "
                     f"Untested entities: {info['untested_entities']}")
        if info["is_utility"]:
            lines.append(f"       UTILITY module, blast radius: {info['blast_radius']} files")

    lines.append("\n" + "=" * 80)
    lines.append("END OF ANALYSIS")
    lines.append("=" * 80 + "\n")

    output = "\n".join(lines)
    print(output)

    OUTPUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    with open(OUTPUT_PATH, "w") as f:
        f.write(output)
    print(f"\nResults saved to {OUTPUT_PATH}")


if __name__ == "__main__":
    main()
