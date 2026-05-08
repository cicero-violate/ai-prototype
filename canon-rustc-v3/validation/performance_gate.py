#!/usr/bin/env python3
"""Hash-bound performance evidence gate."""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
from typing import Any


def canonical_hash(value: object) -> str:
    payload = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(payload).hexdigest()


def read_json(path: pathlib.Path | None) -> dict[str, Any] | None:
    if not path:
        return None
    with path.open(encoding="utf-8") as handle:
        return json.load(handle)


def write_json(path: pathlib.Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def issue(kind: str, detail: str) -> dict[str, str]:
    return {"kind": kind, "detail": detail}


def hash_valid(report: dict[str, Any] | None, field: str) -> bool:
    if not report:
        return False
    digest = report.get(field)
    stable = dict(report)
    stable.pop(field, None)
    return isinstance(digest, str) and digest == canonical_hash(stable)


def number(value: object) -> float | None:
    return float(value) if isinstance(value, (int, float)) else None


def metric(source: dict[str, Any], name: str) -> float | None:
    direct = number(source.get(name))
    if direct is not None:
        return direct
    nested = source.get("performance")
    return number(nested.get(name)) if isinstance(nested, dict) else None


def scale_state(scale: dict[str, Any] | None) -> dict[str, Any]:
    return {
        "report_present": scale is not None,
        "report_hash_valid": hash_valid(scale, "report_hash"),
        "status": (scale or {}).get("status"),
        "node_count": (scale or {}).get("node_count"),
        "edge_count": (scale or {}).get("edge_count"),
        "risk_additions": (scale or {}).get("risk_additions"),
        "prompt_fact_count": (scale or {}).get("prompt_fact_count"),
        "gate_status": (scale or {}).get("gate_status"),
        "elapsed_ms": (scale or {}).get("elapsed_ms"),
        "threshold_ms": (scale or {}).get("threshold_ms"),
    }


def native_state(
    native: dict[str, Any] | None,
    required: bool,
    max_overhead_ratio: float | None,
    max_wrapped_ms: float | None,
) -> dict[str, Any]:
    baseline = metric(native or {}, "baseline_ms")
    wrapped = metric(native or {}, "wrapped_ms")
    ratio = metric(native or {}, "overhead_ratio")
    hash_field = "receipt_hash" if isinstance((native or {}).get("receipt_hash"), str) else "report_hash"
    computed = (wrapped / baseline) if baseline and wrapped is not None else None
    return {
        "required": required,
        "report_present": native is not None,
        "report_hash_valid": hash_valid(native, hash_field),
        "status": (native or {}).get("status"),
        "baseline_ms": baseline,
        "wrapped_ms": wrapped,
        "overhead_ratio": ratio,
        "computed_overhead_ratio": computed,
        "max_overhead_ratio": max_overhead_ratio,
        "max_wrapped_ms": max_wrapped_ms,
    }


def build_report(args: argparse.Namespace) -> dict[str, Any]:
    scale = scale_state(read_json(args.scale_report))
    native = native_state(
        read_json(args.native_overhead_report),
        args.require_native_overhead,
        args.max_overhead_ratio,
        args.max_wrapped_ms,
    )
    missing: list[dict[str, str]] = []
    failures: list[dict[str, str]] = []

    if not scale["report_present"]:
        failures.append(issue("scale_report_absent", "semantic scale report is absent"))
    elif not scale["report_hash_valid"]:
        failures.append(issue("scale_report_hash_invalid", "semantic scale report hash is invalid"))

    elapsed = number(scale["elapsed_ms"])
    threshold = number(scale["threshold_ms"])
    if elapsed is None or threshold is None:
        failures.append(issue("scale_timing_absent", "scale elapsed/threshold timing is absent"))
    elif elapsed > threshold:
        failures.append(issue("scale_threshold_exceeded", "semantic scale elapsed time exceeds threshold"))

    if scale["status"] != "pass":
        failures.append(issue("scale_status_not_pass", "semantic scale report did not pass"))
    if scale["gate_status"] != "deny":
        failures.append(issue("scale_gate_unexpected", "unproved risk delta must remain denied"))

    if not native["report_present"]:
        missing.append(issue("native_overhead_absent", "baseline vs wrapped compiler overhead is absent"))
        if args.require_native_overhead:
            failures.append(issue("native_overhead_required", "required native overhead report is absent"))
    else:
        if not native["report_hash_valid"]:
            failures.append(issue("native_overhead_hash_invalid", "native overhead report hash is invalid"))
        metrics = [native["baseline_ms"], native["wrapped_ms"], native["overhead_ratio"]]
        if not all(isinstance(value, (int, float)) and value > 0 for value in metrics):
            failures.append(issue("native_overhead_metrics_invalid", "native overhead metrics must be positive numbers"))
        computed = native["computed_overhead_ratio"]
        ratio = native["overhead_ratio"]
        if isinstance(computed, float) and isinstance(ratio, float) and abs(computed - ratio) > 0.001:
            failures.append(issue("native_overhead_ratio_mismatch", "overhead_ratio must equal wrapped_ms / baseline_ms"))
        if native["status"] not in {"pass", "pass_with_skip", None}:
            failures.append(issue("native_overhead_status_not_pass", "native overhead report did not pass"))
        max_ratio = native["max_overhead_ratio"]
        if isinstance(max_ratio, float) and isinstance(ratio, float) and ratio > max_ratio:
            failures.append(
                issue(
                    "native_overhead_ratio_exceeded",
                    f"native overhead ratio {ratio:.3f} exceeds threshold {max_ratio:.3f}",
                )
            )
        max_wrapped_ms = native["max_wrapped_ms"]
        wrapped_ms = native["wrapped_ms"]
        if isinstance(max_wrapped_ms, float) and isinstance(wrapped_ms, float) and wrapped_ms > max_wrapped_ms:
            failures.append(
                issue(
                    "native_wrapped_time_exceeded",
                    f"wrapped check {wrapped_ms:.3f} ms exceeds threshold {max_wrapped_ms:.3f} ms",
                )
            )

    report: dict[str, Any] = {
        "schema_version": 1,
        "mode": "performance_gate",
        "status": "fail" if failures else ("pass_with_skip" if missing else "pass"),
        "scale": scale,
        "native_overhead": native,
        "missing_signals": missing,
        "failures": failures,
    }
    report["receipt_hash"] = canonical_hash(report)
    return report


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--scale-report", type=pathlib.Path, required=True)
    parser.add_argument("--native-overhead-report", type=pathlib.Path)
    parser.add_argument("--require-native-overhead", action="store_true")
    parser.add_argument("--max-overhead-ratio", type=float)
    parser.add_argument("--max-wrapped-ms", type=float)
    parser.add_argument("--report", type=pathlib.Path, default=pathlib.Path("validation/performance_report.eval.json"))
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    report = build_report(args)
    write_json(args.report, report)
    print(f"performance gate: {report['status']} missing={len(report['missing_signals'])} failures={len(report['failures'])}")
    return 1 if report["status"] == "fail" else 0


if __name__ == "__main__":
    raise SystemExit(main())