#!/usr/bin/env bash
# Deterministic rustc-artifact recapture/check wrapper for SCORE_REPORT refreshes.
#
# Required environment, when overriding defaults:
#   CANON_RUSTC_V3_ARTIFACT_DIR  Artifact root to validate or recapture.
#                                Default: ../state/rustc, matching .cargo/config.toml.
#   CARGO                        Cargo executable. Default: cargo.
#   TMPDIR                       Rust temporary directory. Default: target/test-tmp.
#   RUSTC_WRAPPER                Optional rustc wrapper override.
#   RUSTC_WORKSPACE_WRAPPER      Optional workspace wrapper override.
#
# Modes:
#   --check      Validate that compatible semantic artifacts already exist.
#   --recapture  Run cargo check to let the configured canon-rustc wrapper recapture artifacts,
#                then validate the resulting artifact root.

set -euo pipefail

usage() {
    cat >&2 <<'USAGE'
usage: scripts/recapture_rustc_graphs.sh [--check|--recapture]

Checks or recaptures canon-rustc-v3 semantic artifacts for score refreshes.
The script does not commit generated artifacts. It fails closed when the wrapper,
toolchain, or artifact root cannot produce compatible judgement-style artifacts.
USAGE
}

mode="${1:---check}"
case "$mode" in
    --check|--recapture) ;;
    -h|--help)
        usage
        exit 0
        ;;
    *)
        usage
        exit 2
        ;;
esac

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"

artifact_root="${CANON_RUSTC_V3_ARTIFACT_DIR:-../state/rustc}"
case "$artifact_root" in
    /*) artifact_abs="$artifact_root" ;;
    *) artifact_abs="$repo_root/$artifact_root" ;;
esac

tmpdir="${TMPDIR:-$repo_root/target/test-tmp}"
cargo_bin="${CARGO:-cargo}"

require_file() {
    if [[ ! -f "$1" ]]; then
        echo "error: required file missing: $1" >&2
        exit 1
    fi
}

validate_artifact_root() {
    local root="$1"

    if [[ ! -d "$root" ]]; then
        echo "error: artifact root does not exist: $root" >&2
        echo "hint: run '$0 --recapture' with the canon-rustc wrapper configured" >&2
        exit 1
    fi

    local manifest_count
    manifest_count=$(find "$root" -mindepth 2 -maxdepth 2 -name manifest.json -type f | wc -l | tr -d ' ')
    if [[ "$manifest_count" -eq 0 ]]; then
        echo "error: no compatible manifest.json files found under: $root" >&2
        echo "hint: expected paths like $root/ai/manifest.json and $root/ai/semantic_index.jsonl" >&2
        exit 1
    fi

    require_file "$root/ai/manifest.json"
    require_file "$root/ai/semantic_index.jsonl"

    python3 - "$root" <<'PY'
import json
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
manifest = root / "ai" / "manifest.json"
semantic = root / "ai" / "semantic_index.jsonl"
data = json.loads(manifest.read_text())
if "crate_target" not in data or "artifacts" not in data:
    raise SystemExit(f"error: incompatible manifest schema in {manifest}")
with semantic.open() as fh:
    first = json.loads(fh.readline())
if first.get("kind") != "semantic_sentinel" or not first.get("schema_version"):
    raise SystemExit(f"error: incompatible semantic index sentinel in {semantic}")
print(f"semantic artifact check: pass root={root}")
PY
}

if [[ "$mode" == "--recapture" ]]; then
    mkdir -p "$tmpdir"
    mkdir -p "$artifact_abs"
    echo "recapture: cargo=$cargo_bin artifact_root=$artifact_abs" >&2
    TMPDIR="$tmpdir" \
    RUSTC_WRAPPER="${RUSTC_WRAPPER:-}" \
    RUSTC_WORKSPACE_WRAPPER="${RUSTC_WORKSPACE_WRAPPER:-}" \
    CANON_RUSTC_V3_ARTIFACT_DIR="$artifact_abs" \
    CANON_RUSTC_V3_ARTIFACT_MODE=full \
        "$cargo_bin" check
fi

validate_artifact_root "$artifact_abs"
