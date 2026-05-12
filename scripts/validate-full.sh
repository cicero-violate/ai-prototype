#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
export LD_LIBRARY_PATH="$(rustc --print sysroot)/lib:${LD_LIBRARY_PATH:-}"

REPORT_DIR="${REPORT_DIR:-state/rustc}"
WRAPPER="$ROOT/canon-rustc-v3/target/debug/canon-rustc-v3"
if [ ! -x "$WRAPPER" ]; then
  echo "== build canon-rustc-v3 wrapper =="
  cargo --config 'build.rustc-wrapper=""' build \
    --manifest-path canon-rustc-v3/Cargo.toml \
    --bin canon-rustc-v3
fi

mkdir -p "$REPORT_DIR"

echo "== cargo build =="
cargo build

echo "== live graph validation =="
python3 canon-rustc-v3/validation/semantic_spine.py --graph state/rustc
python3 canon-rustc-v3/validation/auto_refactor_live.py \
  --artifact-root ../state/rustc \
  --report-dir ../state/rustc/auto-refactor

echo "== cargo test =="
cargo test

echo "== validation smoke/static =="
python3 canon-rustc-v3/validation/schema16_relation_contract.py
python3 canon-rustc-v3/validation/semantic_spine.py

echo "== semantic preflight report =="
python3 canon-rustc-v3/validation/semantic_preflight.py \
  --artifact-root state/rustc \
  --report "$REPORT_DIR/semantic-preflight-step2.json"

echo "== semantic scale report =="
python3 canon-rustc-v3/validation/semantic_scale_probe.py \
  --nodes "${SEMANTIC_SCALE_NODES:-5000}" \
  --fanout "${SEMANTIC_SCALE_FANOUT:-2}" \
  --risk-additions "${SEMANTIC_SCALE_RISK_ADDITIONS:-100}" \
  --threshold-ms "${SEMANTIC_SCALE_THRESHOLD_MS:-2000}" \
  --report "$REPORT_DIR/semantic-scale-step3.json"

echo "== performance gate report =="
python3 canon-rustc-v3/validation/performance_gate.py \
  --scale-report "$REPORT_DIR/semantic-scale-step3.json" \
  --max-overhead-ratio "${MAX_OVERHEAD_RATIO:-3.0}" \
  --max-wrapped-ms "${MAX_WRAPPED_MS:-1000}" \
  --report "$REPORT_DIR/performance-step3.json"

echo "== report verification =="
python3 canon-rustc-v3/validation/semantic_spine.py \
  --preflight-report "$REPORT_DIR/semantic-preflight-step2.json" \
  --performance-report "$REPORT_DIR/performance-step3.json"

echo "full validation: pass"
echo "reports: $REPORT_DIR"
