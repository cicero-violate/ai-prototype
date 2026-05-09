#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from pathlib import Path
from typing import Any


REQUIRED_WORKFLOW_FILES = {
    "old-graph.ndjson",
    "new-graph.ndjson",
    "ops.ndjson",
    "expected-patch.diff",
}
REQUIRED_WORKFLOW_COMMANDS = (
    "verify-ops",
    "generate-patch",
    "verify-landing",
    "verify-receipts",
)
REQUIRED_GENERATED_OUTPUTS = (
    "patch.diff",
    "patch-receipt.ndjson",
    "mutation-receipt.ndjson",
)


def sha256_file(path: Path) -> str | None:
    if not path.exists():
        return None
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git_head(root: Path) -> str:
    done = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=root,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    return done.stdout.strip()


def inspect_graph_workflow_fixture(root: Path) -> dict[str, Any]:
    fixture = root / "tests" / "fixtures" / "graph_mutation_cli_workflow"
    manifest = fixture / "MANIFEST.txt"
    result: dict[str, Any] = {
        "graph_workflow_fixture_present": fixture.exists() and manifest.exists(),
        "graph_workflow_fixture_status": "missing",
        "graph_workflow_fixture_evidence_files": [],
        "graph_workflow_fixture_integrity_valid": False,
        "graph_workflow_fixture_commands_present": False,
        "graph_workflow_fixture_landing_command_present": False,
        "graph_workflow_fixture_ledger_command_present": False,
        "graph_workflow_fixture_receipt_snapshot_present": False,
        "graph_workflow_fixture_command_sequence_valid": False,
        "graph_workflow_fixture_generated_outputs_present": False,
        "graph_workflow_fixture_receipt_ledger_flow_valid": False,
        "graph_workflow_fixture_integrity_rows": [],
        "graph_workflow_fixture_missing": ["MANIFEST.txt"],
    }
    if not fixture.exists() or not manifest.exists():
        return result

    files: list[str] = []
    commands: list[str] = []
    integrity_rows: list[dict[str, str]] = []
    section = ""
    for raw_line in manifest.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if line in {"files:", "commands:", "integrity:"}:
            section = line.rstrip(":")
            continue
        if section == "files" and raw_line.startswith("  "):
            files.append(line)
        elif section == "commands" and raw_line.startswith("  "):
            commands.append(line)
        elif section == "integrity" and raw_line.startswith("  "):
            parts = line.split()
            if len(parts) == 3:
                integrity_rows.append(
                    {"file": parts[0], "algorithm": parts[1], "hash": parts[2].lower()}
                )

    evidence_files = [
        str((fixture / file).relative_to(root)) for file in files if (fixture / file).exists()
    ]
    evidence_files.append(str(manifest.relative_to(root)))
    missing_files = sorted(file for file in files if not (fixture / file).exists())
    missing_required_files = sorted(REQUIRED_WORKFLOW_FILES.difference(files))
    missing_commands = sorted(
        command for command in REQUIRED_WORKFLOW_COMMANDS if not any(command in item for item in commands)
    )
    integrity_failures = [
        row["file"]
        for row in integrity_rows
        if row["algorithm"] != "sha256"
        or not (fixture / row["file"]).exists()
        or sha256_file(fixture / row["file"]) != row["hash"]
    ]
    integrity_valid = bool(integrity_rows) and not integrity_failures
    landing_command_present = any("verify-landing" in command for command in commands)
    ledger_command_present = any("verify-receipts" in command for command in commands)
    commands_present = not missing_commands
    command_kinds = [command.split()[1] for command in commands if len(command.split()) >= 2]
    command_sequence_valid = command_kinds == list(REQUIRED_WORKFLOW_COMMANDS)
    generated_outputs_present = all(
        any(output in command for command in commands) for output in REQUIRED_GENERATED_OUTPUTS
    )
    receipt_ledger_flow_valid = any(
        "verify-receipts" in command
        and "patch-receipt.ndjson" in command
        and "mutation-receipt.ndjson" in command
        for command in commands
    )
    receipt_snapshot_present = (
        integrity_valid
        and commands_present
        and command_sequence_valid
        and generated_outputs_present
        and receipt_ledger_flow_valid
        and landing_command_present
        and ledger_command_present
        and not missing_required_files
        and not missing_files
    )
    missing = sorted(set(missing_files + missing_required_files + missing_commands + integrity_failures))
    result.update(
        {
            "graph_workflow_fixture_status": "pass" if receipt_snapshot_present else "fail",
            "graph_workflow_fixture_evidence_files": sorted(set(evidence_files)),
            "graph_workflow_fixture_integrity_valid": integrity_valid,
            "graph_workflow_fixture_commands_present": commands_present,
            "graph_workflow_fixture_landing_command_present": landing_command_present,
            "graph_workflow_fixture_ledger_command_present": ledger_command_present,
            "graph_workflow_fixture_receipt_snapshot_present": receipt_snapshot_present,
            "graph_workflow_fixture_command_sequence_valid": command_sequence_valid,
            "graph_workflow_fixture_generated_outputs_present": generated_outputs_present,
            "graph_workflow_fixture_receipt_ledger_flow_valid": receipt_ledger_flow_valid,
            "graph_workflow_fixture_integrity_rows": integrity_rows,
            "graph_workflow_fixture_missing": missing,
        }
    )
    return result


def graph_fixture_report(root: Path) -> dict[str, Any]:
    fixture = inspect_graph_workflow_fixture(root)
    status = "pass" if fixture.get("graph_workflow_fixture_receipt_snapshot_present") else "fail"
    return {
        "event": "graph_fixture_report",
        "schema_version": 1,
        "git_head": git_head(root),
        "validation_status": status,
        "graph_evidence_classification_present": True,
        "graph_evidence_status": "graph_mutation_landed_with_receipt_snapshot"
        if status == "pass"
        else "graph_mutation_evidence_contract_missing",
        "graph_evidence_status_options": [
            "graph_mutation_evidence_contract_missing",
            "graph_mutation_landed_with_receipt_snapshot",
        ],
        "graph_fixture_validator": "scripts/validate_graph_workflow_fixture.py",
        **fixture,
        "missing_signal_flags": {
            "missing_graph_workflow_fixture_receipt_snapshot": not fixture.get(
                "graph_workflow_fixture_receipt_snapshot_present", False
            ),
        },
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", default=".")
    parser.add_argument("--report", required=True)
    args = parser.parse_args()
    root = Path(args.root).resolve()
    report = graph_fixture_report(root)
    path = Path(args.report)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return 0 if report["validation_status"] == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())