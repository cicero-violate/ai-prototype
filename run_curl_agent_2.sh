#!/usr/bin/env bash
# Usage: ./run_curl_agent.sh "<domain>" "<metric>" [max_steps]
DOMAIN="${1:-use python here /workspace/ai_sandbox/canon-mini-agent/prototype/state/rustc/canon_rustc_v3/graph.json; reduce cyclomatic complexity on canon-rustc-v3 project please}"
METRIC="${2:-cargo build -- no errors}"
MAX_STEPS="${10:-2}"

curl -sS -X POST http://127.0.0.1:9100/spawn \
  -H "content-type: application/json" \
  -d "{\"domain\":\"$DOMAIN\",\"metric\":\"$METRIC\",\"max_steps\":$MAX_STEPS}"
