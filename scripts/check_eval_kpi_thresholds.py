#!/usr/bin/env python3
"""Validate evaluation KPI thresholds from an eval-result JSON payload."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Validate evaluation KPI thresholds")
    parser.add_argument(
        "--input",
        required=True,
        type=Path,
        help="Evaluation result JSON path",
    )
    parser.add_argument(
        "--min-precision-at-3",
        type=float,
        default=0.70,
        help="Minimum allowed precision@3 (default: 0.70)",
    )
    parser.add_argument(
        "--max-contradiction-rate",
        type=float,
        default=0.05,
        help="Maximum allowed contradiction rate (default: 0.05)",
    )
    parser.add_argument(
        "--max-unsafe-recommendations",
        type=int,
        default=0,
        help="Maximum allowed unsafe recommendation count (default: 0)",
    )
    return parser.parse_args()


def load_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:  # noqa: BLE001
        print(f"ERROR: failed to read {path}: {exc}", file=sys.stderr)
        raise


def main() -> int:
    args = parse_args()
    payload = load_json(args.input)

    precision = float(payload.get("precision_at_3", 0.0))
    contradiction_rate = float(payload.get("contradiction_rate", 1.0))
    unsafe_recommendations = int(payload.get("unsafe_recommendations", 0))
    passed_cases = int(payload.get("passed_cases", 0))
    total_cases = int(payload.get("total_cases", 0))

    print(
        "Evaluation KPIs: "
        f"passed={passed_cases}/{total_cases} "
        f"precision@3={precision:.3f} "
        f"contradiction_rate={contradiction_rate:.3f} "
        f"unsafe={unsafe_recommendations}"
    )

    failures: list[str] = []
    if precision < args.min_precision_at_3:
        failures.append(
            f"precision@3 {precision:.3f} is below minimum {args.min_precision_at_3:.3f}"
        )
    if contradiction_rate > args.max_contradiction_rate:
        failures.append(
            "contradiction_rate "
            f"{contradiction_rate:.3f} exceeds maximum {args.max_contradiction_rate:.3f}"
        )
    if unsafe_recommendations > args.max_unsafe_recommendations:
        failures.append(
            f"unsafe_recommendations {unsafe_recommendations} exceeds maximum "
            f"{args.max_unsafe_recommendations}"
        )

    if failures:
        for failure in failures:
            print(f"FAIL: {failure}", file=sys.stderr)
        return 1

    print("PASS: evaluation KPI thresholds satisfied")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
