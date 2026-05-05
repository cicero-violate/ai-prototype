#!/usr/bin/env python3
"""Hash-bound validation for standalone Rust toolchain archives.

The repo can statically request ``rustc-dev`` in ``rust-toolchain.toml`` while
the supplied offline archive still lacks it or points at a different nightly.
This gate makes that boundary explicit so portable handoffs cannot treat an
arbitrary ``rust-nightly`` tarball as native rustc-private proof.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import re
import tarfile
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]


def canonical_hash(value: object) -> str:
    payload = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(payload).hexdigest()


def write_json(path: pathlib.Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def sha256_file(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def issue(kind: str, detail: str) -> dict[str, str]:
    return {"kind": kind, "detail": detail}


def repo_toolchain() -> dict[str, Any]:
    path = ROOT / "rust-toolchain.toml"
    text = path.read_text(encoding="utf-8") if path.exists() else ""
    channel_match = re.search(r'^\s*channel\s*=\s*"([^"]+)"', text, re.MULTILINE)
    component_match = re.search(r"^\s*components\s*=\s*\[(.*)\]", text, re.MULTILINE)
    components = re.findall(r'"([^"]+)"', component_match.group(1)) if component_match else []
    channel = channel_match.group(1) if channel_match else None
    date_match = re.fullmatch(r"nightly-(\d{4}-\d{2}-\d{2})", channel or "")
    return {
        "path": str(path.relative_to(ROOT)),
        "channel": channel,
        "pinned_date": date_match.group(1) if date_match else None,
        "components": sorted(components),
    }


def archive_prefix_members(path: pathlib.Path, max_scanned_members: int = 100_000) -> dict[str, Any]:
    """Return bounded top-level-ish archive members using Python tarfile only.

    Rust toolchain archives can place large component payloads early in the
    stream.  The gate therefore bounds total scanned members, not only accepted
    prefix-level entries, so an archive that starts with thousands of deep docs
    cannot turn archive validation into an unbounded tar walk.  The default is
    still high enough to scan the supplied standalone archives completely before
    declaring required components absent.  It does not trust shell ``tar``
    output or extract archive contents.
    """

    members: list[str] = []
    manifest_files: dict[str, str] = {}
    scanned = 0
    with tarfile.open(path, "r|gz") as archive:
        for info in archive:
            scanned += 1
            member = info.name.strip()
            if member.count("/") <= 2:
                members.append(member)
            if member.rsplit("/", 1)[-1] in {"components", "version"} and info.isfile():
                payload = archive.extractfile(info)
                if payload is not None:
                    manifest_files[member] = payload.read(1024 * 1024).decode("utf-8", errors="replace")
            if scanned >= max_scanned_members:
                break
    return {
        "members": members,
        "scanned_members": scanned,
        "max_scanned_members": max_scanned_members,
        "scan_complete": scanned < max_scanned_members,
        "manifest_files": manifest_files,
    }


def archive_state(path: pathlib.Path | None) -> dict[str, Any]:
    if path is None:
        return {"path": None, "present": False, "readable": False}
    state: dict[str, Any] = {
        "path": str(path),
        "present": path.exists(),
        "readable": False,
    }
    if not path.exists():
        return state

    state["sha256"] = sha256_file(path)
    root = path.name.removesuffix(".tar.gz")
    try:
        sample = archive_prefix_members(path)
    except tarfile.TarError as error:
        state["tar_error"] = str(error)
        return state
    members = sample["members"]
    manifest_files = sample["manifest_files"]
    components = sorted(
        {
            parts[1]
            for member in members
            if (parts := member.split("/")) and len(parts) >= 2 and parts[0] == root and parts[1]
        }
    )
    manifest_components = sorted(
        {
            line.strip()
            for member, text in manifest_files.items()
            if member.endswith("/components")
            for line in text.splitlines()
            if line.strip()
        }
    )
    if manifest_components:
        components = manifest_components
    version_text = next((text.strip() for member, text in manifest_files.items() if member.endswith("/version")), "")
    date_match = re.search(r"nightly-(\d{4}-\d{2}-\d{2})", path.name)
    version_date_match = re.search(r"nightly-(\d{4}-\d{2}-\d{2})", version_text)
    state.update(
        {
            "readable": True,
            "components": components,
            "inspection": {
                "method": "python_tarfile_stream",
                "shell_tar_used": False,
                "scanned_members": sample["scanned_members"],
                "max_scanned_members": sample["max_scanned_members"],
                "sampled_prefix_members": len(members),
                "scan_complete": sample["scan_complete"],
            },
            "version": version_text or None,
            "date": (version_date_match or date_match).group(1) if (version_date_match or date_match) else None,
        }
    )
    return state


def build_report(args: argparse.Namespace) -> dict[str, Any]:
    repo = repo_toolchain()
    archive = archive_state(args.archive)
    missing: list[dict[str, str]] = []
    failures: list[dict[str, str]] = []

    if not archive.get("present"):
        missing.append(issue("toolchain_archive_absent", "standalone Rust toolchain archive was not supplied"))
    elif not archive.get("readable"):
        failures.append(issue("toolchain_archive_unreadable", "standalone Rust toolchain archive could not be read"))
    else:
        archive_components = set(archive.get("components", []))
        required_components = set(repo.get("components", []))
        missing_components = sorted(required_components - archive_components)
        if missing_components:
            failures.append(
                issue(
                    "toolchain_archive_missing_components",
                    "archive lacks required components: " + ", ".join(missing_components),
                )
            )
        if "cargo" not in archive_components:
            failures.append(issue("toolchain_archive_missing_cargo", "archive lacks cargo component"))
        if "rustc" not in archive_components:
            failures.append(issue("toolchain_archive_missing_rustc", "archive lacks rustc component"))
        if args.require_channel_match and repo.get("pinned_date") != archive.get("date"):
            failures.append(
                issue(
                    "toolchain_archive_channel_mismatch",
                    f"repo channel date {repo.get('pinned_date')} != archive date {archive.get('date')}",
                )
            )

    report: dict[str, Any] = {
        "schema_version": 1,
        "mode": "toolchain_archive_gate",
        "status": "fail" if failures else ("pass_with_skip" if missing else "pass"),
        "repo_toolchain": repo,
        "archive": archive,
        "requirements": {"channel_match": args.require_channel_match},
        "missing_signals": missing,
        "failures": failures,
    }
    report["receipt_hash"] = canonical_hash({key: value for key, value in report.items() if key != "receipt_hash"})
    return report


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--archive", type=pathlib.Path)
    parser.add_argument("--require-channel-match", action="store_true")
    parser.add_argument("--report", type=pathlib.Path, default=pathlib.Path("validation/toolchain_archive_report.eval.json"))
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    report = build_report(args)
    write_json(args.report, report)
    print(
        "toolchain archive gate: "
        f"{report['status']} missing={len(report['missing_signals'])} failures={len(report['failures'])}"
    )
    return 1 if report["status"] == "fail" else 0


if __name__ == "__main__":
    raise SystemExit(main())
