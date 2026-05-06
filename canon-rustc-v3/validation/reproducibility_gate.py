#!/usr/bin/env python3
"""Hash-bound reproducibility gate for rustc-wrapper handoff."""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import re
import subprocess
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]


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


def run(argv: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(argv, cwd=ROOT, text=True, capture_output=True, check=False)


def channel() -> str | None:
    path = ROOT / "rust-toolchain.toml"
    if not path.exists():
        return None
    match = re.search(r'^\s*channel\s*=\s*"([^"]+)"', path.read_text(encoding="utf-8"), re.MULTILINE)
    return match.group(1) if match else None


def components() -> list[str]:
    path = ROOT / "rust-toolchain.toml"
    if not path.exists():
        return []
    text = path.read_text(encoding="utf-8")
    match = re.search(r'^\s*components\s*=\s*\[(.*)\]', text, re.MULTILINE)
    if not match:
        return []
    return re.findall(r'"([^"]+)"', match.group(1))


def cargo_config_state() -> dict[str, Any]:
    path = ROOT / ".cargo" / "config.toml"
    text = path.read_text(encoding="utf-8") if path.exists() else ""
    match = re.search(r'^\s*rustc\s*=\s*"([^"]+)"', text, re.MULTILINE)
    return {
        "present": path.exists(),
        "forced_rustc": match is not None,
        "forced_rustc_value": match.group(1) if match else None,
    }


def submodule_initialized() -> bool:
    status = run(["git", "submodule", "status", "--", "vendor/rust-source"])
    line = status.stdout.strip()
    return status.returncode == 0 and bool(line) and not line.startswith("-")


def git_head() -> str | None:
    result = run(["git", "rev-parse", "HEAD"])
    return result.stdout.strip() if result.returncode == 0 else None


def issue(kind: str, detail: str) -> dict[str, str]:
    return {"kind": kind, "detail": detail}


def receipt_valid(report: dict[str, Any] | None) -> bool:
    if not report:
        return False
    receipt = report.get("receipt_hash")
    stable = dict(report)
    stable.pop("receipt_hash", None)
    return isinstance(receipt, str) and receipt == canonical_hash(stable)


def live_state(preflight: dict[str, Any] | None, witness: dict[str, Any] | None, required: bool) -> dict[str, Any]:
    deterministic = (preflight or {}).get("determinism_inputs", {})
    performance = (witness or {}).get("performance")
    return {
        "required": required,
        "preflight_report_present": preflight is not None,
        "preflight_receipt_valid": receipt_valid(preflight),
        "preflight_status": (preflight or {}).get("status"),
        "witness_report_present": witness is not None,
        "witness_receipt_valid": receipt_valid(witness),
        "witness_status": (witness or {}).get("status"),
        "graph_roots_comparable": bool(deterministic.get("graph_roots_comparable")),
        "replay_comparison": deterministic.get("replay_comparison", "not_run"),
        "wrapper_overhead_measured": isinstance(performance, dict)
        and isinstance(performance.get("overhead_ratio"), (int, float)),
    }


def build_report(args: argparse.Namespace) -> dict[str, Any]:
    ch = channel()
    cargo_config = cargo_config_state()
    toolchain = {
        "channel": ch,
        "components": components(),
        "pinned": bool(ch and ch != "nightly"),
        "drift_risk": ch in {None, "nightly"},
    }
    lockfile = {"present": (ROOT / "Cargo.lock").exists(), "required": args.require_lockfile}
    submodule = {
        "path": "vendor/rust-source",
        "configured": (ROOT / ".gitmodules").read_text(encoding="utf-8").find("vendor/rust-source") >= 0,
        "initialized": submodule_initialized(),
        "required": args.require_submodule,
    }
    live = live_state(read_json(args.preflight_report), read_json(args.witness_report), args.require_live_validation)

    missing: list[dict[str, str]] = []
    failures: list[dict[str, str]] = []

    if toolchain["drift_risk"]:
        missing.append(issue("unpinned_toolchain", "rust-toolchain.toml is absent or uses bare nightly"))
    if cargo_config["forced_rustc"]:
        missing.append(issue("forced_rustc_config", ".cargo/config.toml forces a host-specific rustc binary"))
    if not lockfile["present"]:
        missing.append(issue("cargo_lock_absent", "Cargo.lock is absent"))
    if not submodule["initialized"]:
        missing.append(issue("rust_source_uninitialized", "vendor/rust-source is not initialized"))
    if not live["preflight_report_present"]:
        missing.append(issue("preflight_report_absent", "semantic preflight report was not supplied"))
    if not live["witness_report_present"]:
        missing.append(issue("witness_report_absent", "semantic witness report was not supplied"))
    if not (live["graph_roots_comparable"] and live["replay_comparison"] == "passed"):
        missing.append(issue("live_replay_unproven", "comparable live graph replay did not pass"))
    if not live["wrapper_overhead_measured"]:
        missing.append(issue("wrapper_overhead_unmeasured", "baseline/wrapped compiler overhead is absent"))

    if args.require_lockfile and not lockfile["present"]:
        failures.append(issue("cargo_lock_required", "required Cargo.lock is absent"))
    if args.require_portable_config and cargo_config["forced_rustc"]:
        failures.append(issue("portable_config_required", "required portable cargo config still forces rustc"))
    if args.require_submodule and not submodule["initialized"]:
        failures.append(issue("rust_source_required", "required vendor/rust-source is not initialized"))
    if args.require_live_validation and not (
        live["preflight_receipt_valid"]
        and live["witness_receipt_valid"]
        and live["graph_roots_comparable"]
        and live["replay_comparison"] == "passed"
        and live["wrapper_overhead_measured"]
    ):
        failures.append(issue("live_validation_required", "required live validation evidence is incomplete"))

    report: dict[str, Any] = {
        "schema_version": 1,
        "mode": "reproducibility_gate",
        "status": "fail" if failures else ("pass_with_skip" if missing else "pass"),
        "head_commit": git_head(),
        "cargo_config": cargo_config,
        "toolchain": toolchain,
        "lockfile": lockfile,
        "submodule": submodule,
        "live_validation": live,
        "missing_signals": missing,
        "failures": failures,
    }
    report["receipt_hash"] = canonical_hash(report)
    return report


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--preflight-report", type=pathlib.Path)
    parser.add_argument("--witness-report", type=pathlib.Path)
    parser.add_argument("--require-lockfile", action="store_true")
    parser.add_argument("--require-portable-config", action="store_true")
    parser.add_argument("--require-submodule", action="store_true")
    parser.add_argument("--require-live-validation", action="store_true")
    parser.add_argument("--report", type=pathlib.Path, default=pathlib.Path("validation/reproducibility_report.eval.json"))
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    report = build_report(args)
    write_json(args.report, report)
    print(f"reproducibility gate: {report['status']} missing={len(report['missing_signals'])} failures={len(report['failures'])}")
    return 1 if report["status"] == "fail" else 0


if __name__ == "__main__":
    raise SystemExit(main())