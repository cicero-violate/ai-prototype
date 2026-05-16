#!/usr/bin/env bash
# Usage: ./run_curl_agent.sh "<domain>" "<metric>" [max_steps]
DOMAIN="${1:-perform semantic normalization or concept consolidation on the ai codebase, refactoring toward a canonical domain model using ubiquitous language}"
METRIC="${2:-cargo build -- no errors}"
MAX_STEPS="${3:-2}"

curl -sS -X POST http://127.0.0.1:9100/spawn \
  -H "content-type: application/json" \
  -d "{\"domain\":\"$DOMAIN\",\"metric\":\"$METRIC\",\"max_steps\":$MAX_STEPS}"
