#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

REPORT_DIR="${REPORT_DIR:-validation/reports}"
mkdir -p "$REPORT_DIR"

echo "== cargo build =="
cargo build

echo "== cargo test =="
cargo test

echo "== validation smoke/static =="
python3 validation/schema16_relation_contract.py
python3 validation/auto_refactor_surface_smoke.py
python3 validation/auto_refactor_ops_smoke.py
python3 validation/semantic_spine.py

echo "== semantic witness report =="
python3 validation/run_semantic_witness.py \
  --require-cargo \
  --report "$REPORT_DIR/semantic-witness-step2.json"

echo "== semantic preflight report =="
python3 validation/semantic_preflight.py \
  --artifact-root state/rustc-test-1 \
  --compare-artifact-root state/rustc-test-2 \
  --require-live-replay \
  --report "$REPORT_DIR/semantic-preflight-step2.json"

echo "== semantic scale report =="
python3 validation/semantic_scale_probe.py \
  --nodes "${SEMANTIC_SCALE_NODES:-5000}" \
  --fanout "${SEMANTIC_SCALE_FANOUT:-2}" \
  --risk-additions "${SEMANTIC_SCALE_RISK_ADDITIONS:-100}" \
  --threshold-ms "${SEMANTIC_SCALE_THRESHOLD_MS:-2000}" \
  --report "$REPORT_DIR/semantic-scale-step3.json"

echo "== performance gate report =="
python3 validation/performance_gate.py \
  --scale-report "$REPORT_DIR/semantic-scale-step3.json" \
  --native-overhead-report "$REPORT_DIR/semantic-witness-step2.json" \
  --require-native-overhead \
  --max-overhead-ratio "${MAX_OVERHEAD_RATIO:-3.0}" \
  --max-wrapped-ms "${MAX_WRAPPED_MS:-1000}" \
  --report "$REPORT_DIR/performance-step3.json"

echo "== report verification =="
python3 validation/semantic_spine.py \
  --preflight-report "$REPORT_DIR/semantic-preflight-step2.json" \
  --performance-report "$REPORT_DIR/performance-step3.json"

echo "full validation: pass"
echo "reports: $REPORT_DIR"
