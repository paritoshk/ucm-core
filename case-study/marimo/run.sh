#!/usr/bin/env bash
# UCM vs marimo Case Study — Full Pipeline
# Usage: ./run.sh [marimo_path] [ucm_binary]
#
# Prerequisites:
#   git clone --depth 1 https://github.com/marimo-team/marimo ~/marimo
#   cargo build --release  (in UCM repo root)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
MARIMO_PATH="${1:-$HOME/marimo}"
UCM="${2:-$SCRIPT_DIR/../../target/release/ucm}"
RESULTS_DIR="$SCRIPT_DIR/results"
PACKAGE_ROOT="marimo"
SCAN_DIR="$MARIMO_PATH/marimo"
FLAGS="--language python --no-limit --package-root $PACKAGE_ROOT"

# Validate prerequisites
if [ ! -f "$UCM" ]; then
    echo "ERROR: UCM binary not found at $UCM"
    echo "  Run: cargo build --release"
    exit 1
fi
if [ ! -d "$SCAN_DIR" ]; then
    echo "ERROR: marimo source not found at $SCAN_DIR"
    echo "  Run: git clone --depth 1 https://github.com/marimo-team/marimo $MARIMO_PATH"
    exit 1
fi

mkdir -p "$RESULTS_DIR"

echo "============================================="
echo "UCM vs marimo Case Study"
echo "============================================="
echo "  marimo:  $SCAN_DIR"
echo "  UCM:     $UCM"
echo ""

# ── Step 1: Scan ──
echo "── Step 1: Scanning marimo ──"
"$UCM" scan "$SCAN_DIR" $FLAGS 2>&1 | tee "$RESULTS_DIR/scan-output.txt"
echo ""

# ── Step 2: Export graph ──
echo "── Step 2: Exporting graph as JSON ──"
"$UCM" graph "$SCAN_DIR" $FLAGS --export json > "$RESULTS_DIR/graph.json" 2>/dev/null
echo "  Exported to results/graph.json ($(du -h "$RESULTS_DIR/graph.json" | cut -f1))"
echo ""

# ── Step 3: Impact analysis scenarios ──
echo "── Step 3: Impact Analysis (5 scenarios) ──"

run_impact() {
    local label="$1" file="$2" symbol="$3" outfile="$4"
    "$UCM" impact "$file" "$symbol" --path "$SCAN_DIR" $FLAGS --json \
        > "$RESULTS_DIR/$outfile" 2>/dev/null || true
    local result
    result=$(python3 -c "
import json, sys
try:
    d=json.load(open('$RESULTS_DIR/$outfile'))
    di=len(d.get('direct_impacts',[]))
    ii=len(d.get('indirect_impacts',[]))
    print(f'{di} direct, {ii} indirect')
except: print('empty (entity not in graph)')
" 2>/dev/null || echo "parse error")
    echo "  [$label] $file#$symbol → $result"
}

run_impact "A" "_runtime/executor.py" "Executor.execute_cell" "impact-A-runtime.json"
run_impact "B" "_ast/visitor.py"      "ScopedVisitor"          "impact-B-ast.json"
run_impact "C" "_runtime/dataflow/graph.py" "DirectedGraph"    "impact-C-graph.json"
run_impact "D" "_plugins/ui/_impl/input.py" "slider"           "impact-D-ui.json"
run_impact "E" "_utils/flatten.py"    "flatten"                "impact-E-util.json"

echo ""

# ── Step 4: Test intent ──
echo "── Step 4: Test Intent (Scenario A) ──"
"$UCM" intent "_runtime/executor.py" "Executor.execute_cell" \
    --path "$SCAN_DIR" $FLAGS --json \
    > "$RESULTS_DIR/intent-A-runtime.json" 2>/dev/null || true
python3 -c "
import json
try:
    d=json.load(open('$RESULTS_DIR/intent-A-runtime.json'))
    s=d['summary']
    print(f'  Scenarios: {s[\"total_scenarios\"]} (high={s[\"high_count\"]}, med={s[\"medium_count\"]}, low={s[\"low_count\"]})')
    print(f'  Risks: {len(d[\"risks\"])}, Coverage gaps: {len(d[\"coverage_gaps\"])}')
except: print('  (no results)')
" 2>/dev/null

echo ""

# ── Step 5: Graph analysis ──
echo "── Step 5: Graph Statistics ──"
python3 "$SCRIPT_DIR/analyze_graph.py" "$RESULTS_DIR/graph.json" 2>/dev/null | head -30

echo ""
echo "============================================="
echo "Done. Results in: $RESULTS_DIR/"
echo "============================================="
ls -1 "$RESULTS_DIR/"
