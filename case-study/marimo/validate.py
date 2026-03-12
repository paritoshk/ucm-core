#!/usr/bin/env python3
"""Validate UCM impact analysis against ground truth.

Usage: python3 validate.py results/impact-A-runtime.json ground-truth.json scenario_A
"""

import json
import sys


def load_json(path):
    with open(path) as f:
        return json.load(f)


def extract_entity_names(impact_report):
    """Extract all entity names from UCM impact report."""
    names = set()
    for impact in impact_report.get("direct_impacts", []):
        names.add(impact.get("name", ""))
    for impact in impact_report.get("indirect_impacts", []):
        names.add(impact.get("name", ""))
    return names


def compute_metrics(predicted: set, actual: set):
    """Compute precision, recall, F1."""
    if not predicted and not actual:
        return {"precision": 1.0, "recall": 1.0, "f1": 1.0}

    tp = len(predicted & actual)
    fp = len(predicted - actual)
    fn = len(actual - predicted)

    precision = tp / (tp + fp) if (tp + fp) > 0 else 0.0
    recall = tp / (tp + fn) if (tp + fn) > 0 else 0.0
    f1 = 2 * precision * recall / (precision + recall) if (precision + recall) > 0 else 0.0

    return {
        "precision": precision,
        "recall": recall,
        "f1": f1,
        "true_positives": tp,
        "false_positives": fp,
        "false_negatives": fn,
        "predicted_count": len(predicted),
        "actual_count": len(actual),
    }


def main():
    if len(sys.argv) < 4:
        print("Usage: python3 validate.py <impact.json> <ground-truth.json> <scenario_key>")
        sys.exit(1)

    impact_path = sys.argv[1]
    ground_truth_path = sys.argv[2]
    scenario_key = sys.argv[3]

    impact = load_json(impact_path)
    ground_truth = load_json(ground_truth_path)

    if scenario_key not in ground_truth:
        print(f"Scenario '{scenario_key}' not found in ground truth")
        print(f"Available: {list(ground_truth.keys())}")
        sys.exit(1)

    predicted = extract_entity_names(impact)
    actual = set(ground_truth[scenario_key].get("expected_impacts", []))

    metrics = compute_metrics(predicted, actual)

    print(f"Scenario: {scenario_key}")
    print(f"  Predicted impacts: {metrics['predicted_count']}")
    print(f"  Actual impacts:    {metrics['actual_count']}")
    print(f"  True positives:    {metrics['true_positives']}")
    print(f"  False positives:   {metrics['false_positives']}")
    print(f"  False negatives:   {metrics['false_negatives']}")
    print(f"  Precision:         {metrics['precision']:.1%}")
    print(f"  Recall:            {metrics['recall']:.1%}")
    print(f"  F1:                {metrics['f1']:.1%}")
    print()

    if metrics["false_positives"] > 0:
        fps = predicted - actual
        print("  False positives (UCM predicted but not in ground truth):")
        for name in sorted(fps)[:10]:
            print(f"    - {name}")

    if metrics["false_negatives"] > 0:
        fns = actual - predicted
        print("  False negatives (in ground truth but UCM missed):")
        for name in sorted(fns)[:10]:
            print(f"    - {name}")


if __name__ == "__main__":
    main()
