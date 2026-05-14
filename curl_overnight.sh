#!/usr/bin/env bash
# run_overnight.sh
DOMAIN="${1:-improve score.md}"
METRIC="${2:-cargo test passes, git add & git commit}"
MAX_STEPS="${3:-20}"
PAUSE="${PAUSE:-15}"  # seconds between spawns

i=0
while true; do
    i=$((i + 1))
    echo "=== spawn $i  $(date) ==="
    bash "$(dirname "$0")/curl.sh" "$DOMAIN" "$METRIC" "$MAX_STEPS" || true
    echo "=== done $i  sleeping ${PAUSE}s ==="
    sleep "$PAUSE"
done
