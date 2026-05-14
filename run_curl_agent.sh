#!/usr/bin/env bash
# Usage: ./run_curl_agent.sh "<domain>" "<metric>" [max_steps]
DOMAIN="${1:-improve score.md}"
METRIC="${2:-cargo test passes, git add & git commit}"
MAX_STEPS="${3:-4}"

curl -sS -X POST http://127.0.0.1:9100/spawn \
  -H "content-type: application/json" \
  -d "{\"domain\":\"$DOMAIN\",\"metric\":\"$METRIC\",\"max_steps\":$MAX_STEPS}"
