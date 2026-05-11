#!/usr/bin/env bash
# Run the agent in loop mode against the running supervisor + MCP connector.
# The supervisor must already be running (see run_supervisor.sh).
#
# Usage: ./run.sh
# Override any variable inline: SUPERVISOR_PORT=9100 ./run.sh

set -euo pipefail

cargo build && cargo build --release
SUPERVISOR_PORT="${SUPERVISOR_PORT:-9100}"

# Resolve worker port from supervisor unless explicitly provided
if [[ -z "${AI_WORKER_PORT:-}" ]]; then
    health=$(curl -sf "http://127.0.0.1:${SUPERVISOR_PORT}/health" 2>/dev/null) || {
        echo "error: supervisor not reachable on port ${SUPERVISOR_PORT} — run run_supervisor.sh first" >&2
        exit 1
    }
    AI_WORKER_PORT=$(echo "$health" | grep -o '"worker_port":[0-9]*' | grep -o '[0-9]*$')
    if [[ -z "$AI_WORKER_PORT" ]]; then
        echo "error: could not parse worker_port from supervisor health response" >&2
        exit 1
    fi
    echo "agent: using worker_port=${AI_WORKER_PORT} (from supervisor)"
fi

AI_WORKER_PORT="$AI_WORKER_PORT" \
SUPERVISOR_PORT="$SUPERVISOR_PORT" \
PROJECT_DIR="${PROJECT_DIR:-/workspace/ai_sandbox/canon-mini-agent/prototype/ai}" \
CANON_OPENAI_BASE_URL="${CANON_OPENAI_BASE_URL:-http://127.0.0.1:8081/v1}" \
MCP_CONNECTOR_URL="${MCP_CONNECTOR_URL:-http://127.0.0.1:4000}" \
./target/release/agent
