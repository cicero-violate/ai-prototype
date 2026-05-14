#!/usr/bin/env bash
# Deterministic graph-artifact recapture/check wrapper for SCORE_REPORT refreshes.
#
# Required environment, when overriding defaults:
#   CANON_RUSTC_V3_ARTIFACT_DIR  Graph artifact root to validate or recapture.
#                                Default: ../state/rustc, matching .cargo/config.toml.
#   CARGO                        Cargo executable. Default: cargo.
#   TMPDIR                       Rust temporary directory. Default: target/test-tmp.
#   RUSTC_WRAPPER                Optional rustc wrapper override.
#   RUSTC_WORKSPACE_WRAPPER      Optional workspace wrapper override.
#
# Modes:
#   --check      Validate that compatible graph artifacts already exist.
#   --recapture  Run cargo check to let the configured canon-rustc wrapper recapture graphs,
#                then validate the resulting artifact root.

set -euo pipefail

usage() {
    cat >&2 <<'USAGE'
usage: scripts/recapture_rustc_graphs.sh [--check|--recapture]

Checks or recaptures canon-rustc-v3 graph artifacts for score refreshes.
The script does not commit generated graphs. It fails closed when the wrapper,
toolchain, or artifact root cannot produce compatible graph.json files.
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

validate_graph_root() {
    local root="$1"

    if [[ ! -d "$root" ]]; then
        echo "error: graph artifact root does not exist: $root" >&2
        echo "hint: run '$0 --recapture' with the canon-rustc wrapper configured" >&2
        exit 1
    fi

    local graph_count
    graph_count=$(find "$root" -mindepth 2 -maxdepth 2 -name graph.json -type f | wc -l | tr -d ' ')
    if [[ "$graph_count" -eq 0 ]]; then
        echo "error: no compatible graph.json files found under: $root" >&2
        echo "hint: expected paths like $root/ai/graph.json and $root/root_validate__bin/graph.json" >&2
        exit 1
    fi

    require_file "$root/ai/graph.json"
    require_file "$root/root_validate__bin/graph.json"

    python3 - "$root" <<'PY'
import json
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
required = [root / "ai" / "graph.json", root / "root_validate__bin" / "graph.json"]
for path in required:
    data = json.loads(path.read_text())
    if "meta" not in data or "nodes" not in data or "edges" not in data:
        raise SystemExit(f"error: incompatible graph schema in {path}")
    meta = data["meta"]
    if not meta.get("crate_name"):
        raise SystemExit(f"error: graph meta.crate_name missing in {path}")
    if not meta.get("schema_version"):
        raise SystemExit(f"error: graph meta.schema_version missing in {path}")
    if not meta.get("graph_hash"):
        raise SystemExit(f"error: graph meta.graph_hash missing in {path}")
print(f"graph artifact check: pass root={root} required={len(required)}")
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
        "$cargo_bin" check
fi

validate_graph_root "$artifact_abs"