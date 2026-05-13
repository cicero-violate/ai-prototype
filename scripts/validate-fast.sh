#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
export LD_LIBRARY_PATH="$(rustc --print sysroot)/lib:${LD_LIBRARY_PATH:-}"

WRAPPER="$ROOT/canon-rustc-v3/target/debug/canon-rustc-v3"
if [ ! -x "$WRAPPER" ]; then
  echo "== build canon-rustc-v3 wrapper =="
  cargo --config 'build.rustc-wrapper=""' build \
    --manifest-path canon-rustc-v3/Cargo.toml \
    --bin canon-rustc-v3
fi

mkdir -p state/rustc

echo "== cargo fmt --check =="
cargo fmt --check

echo "== cargo check =="
cargo check

echo "== live graph validation =="
python3 canon-rustc-v3/validation/semantic_spine.py --graph state/rustc
python3 canon-rustc-v3/validation/auto_refactor_live.py \
  --artifact-root ../state/rustc \
  --report-dir ../state/rustc/auto-refactor

echo "== python smoke validation =="
python3 canon-rustc-v3/validation/schema16_relation_contract.py
python3 canon-rustc-v3/validation/semantic_spine.py

echo "== score report =="
cargo run --manifest-path ../score/Cargo.toml --quiet -- \
  --artifact-root state/rustc \
  --report SCORE_REPORT.md \
  --date "$(date +%Y-%m-%d)"
git add SCORE_REPORT.md

echo "fast validation: pass"
