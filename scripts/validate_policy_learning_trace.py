#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

TRACE_FUNCTION = "learning_policy_llm_feedback_loop_drives_judgment"
REQUIRED = {
    "learning_policy_llm_feedback_loop_drives_judgment": ["src/lib.rs"],
    "PolicyPromotion": ["src/capability/learning/promote.rs"],
    "from_tlog": ["src/capability/learning/promote.rs"],
    "Evidence::PolicyPromotion": ["src/capability/learning/promote.rs", "src/lib.rs"],
    "DistillationRow": ["src/capability/learning/promote.rs"],
    "DISTILLATION_ROW_SCHEMA_VERSION": ["src/capability/learning/promote.rs"],
    "distillation_row_contract": ["src/lib.rs", "src/capability/learning/promote.rs"],
}
FORBIDDEN_IN_POLICY_STORE = ("pub fn promote_feedback", "pub fn promote_feedback_durable")


def read(root: Path, rel: str) -> str:
    path = root / rel
    return path.read_text(encoding="utf-8") if path.exists() else ""


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", default=".")
    parser.add_argument("--report", required=True)
    args = parser.parse_args()
    root = Path(args.root).resolve()

    missing: list[str] = []
    checks = []
    for name, files in REQUIRED.items():
        present = any(name in read(root, rel) for rel in files)
        if name == "distillation_row_contract":
            present = "DistillationRow" in read(root, "src/capability/learning/promote.rs") and "DISTILLATION_ROW_SCHEMA_VERSION" in read(root, "src/capability/learning/promote.rs")
        checks.append({"name": name, "present": present})
        if not present:
            missing.append(name)

    policy_store = read(root, "src/capability/policy/store.rs")
    for token in FORBIDDEN_IN_POLICY_STORE:
        if token in policy_store:
            missing.append(f"forbidden:{token}")
            checks.append({"name": f"forbidden:{token}", "present": False})

    data = {
        "status": "pass" if not missing else "fail",
        "trace_function": TRACE_FUNCTION,
        "missing": missing,
        "missing_count": len(missing),
        "checks": checks,
    }
    report = Path(args.report)
    report.parent.mkdir(parents=True, exist_ok=True)
    report.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return 0 if not missing else 1


if __name__ == "__main__":
    raise SystemExit(main())
