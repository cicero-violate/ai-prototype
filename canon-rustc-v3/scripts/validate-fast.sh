#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "== cargo fmt --check =="
cargo fmt --check

echo "== cargo check =="
cargo check

echo "== live self graph validation =="
python3 validation/semantic_spine.py --graph state/rustc
python3 validation/auto_refactor_live.py --artifact-root state/rustc --graph-editor-root "${GRAPH_EDITOR_ROOT:-../graph-editor}"

echo "== python smoke validation =="
python3 validation/schema16_relation_contract.py
python3 validation/semantic_spine.py

echo "fast validation: pass"
