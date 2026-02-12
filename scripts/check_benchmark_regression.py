#!/usr/bin/env python3
"""Check benchmark regression against a baseline throughput metric."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Validate benchmark regression threshold")
    parser.add_argument("--baseline", required=True, type=Path, help="Baseline benchmark JSON")
    parser.add_argument("--current", required=True, type=Path, help="Current benchmark JSON")
    parser.add_argument(
        "--max-regression",
        type=float,
        default=0.15,
        help="Maximum allowed throughput regression ratio (default: 0.15)",
    )
    return parser.parse_args()


def load_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:  # noqa: BLE001
        print(f"ERROR: failed to read {path}: {exc}", file=sys.stderr)
        raise


def throughput(payload: dict, path: Path) -> float:
    value = payload.get("avg_throughput_mb_s")
    if value is None:
        raise ValueError(f"avg_throughput_mb_s missing in {path}")
    try:
        return float(value)
    except ValueError as exc:
        raise ValueError(f"invalid avg_throughput_mb_s in {path}: {value}") from exc


def main() -> int:
    args = parse_args()

    baseline_payload = load_json(args.baseline)
    current_payload = load_json(args.current)

    baseline = throughput(baseline_payload, args.baseline)
    current = throughput(current_payload, args.current)

    if baseline <= 0:
        print("ERROR: baseline throughput must be > 0", file=sys.stderr)
        return 2

    min_allowed = baseline * (1.0 - args.max_regression)
    regression = (baseline - current) / baseline

    print(
        f"Benchmark throughput baseline={baseline:.3f} MB/s current={current:.3f} MB/s "
        f"max_regression={args.max_regression:.3f}"
    )

    if current < min_allowed:
        print(
            f"FAIL: throughput regression {regression:.3%} exceeds allowed {args.max_regression:.3%}",
            file=sys.stderr,
        )
        return 1

    print("PASS: benchmark regression check within threshold")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
