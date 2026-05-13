#!/usr/bin/env bash
# Run the agent in loop mode against the running supervisor + MCP connector.
# The supervisor must already be running (see run_supervisor.sh).
#
# Usage: ./run.sh
# Override any variable inline: SUPERVISOR_PORT=9100 ./run.sh

set -euo pipefail

cargo build && cargo build --release
SUPERVISOR_PORT="${SUPERVISOR_PORT:-9100}"

# Resolve the active worker port from supervisor. AI_WORKER_PORT is only a
# fallback for a 204 reload response; do not trust a stale inherited value over
# supervisor's active worker.
reload=$(curl -sf -X POST "http://127.0.0.1:${SUPERVISOR_PORT}/reload" 2>/dev/null) || {
    echo "error: supervisor not reachable on port ${SUPERVISOR_PORT} — run run_supervisor.sh first" >&2
    exit 1
}

resolved_worker_port=$(echo "$reload" | grep -o '"worker_port":[0-9]*' | grep -o '[0-9]*$' | head -n 1)
if [[ -n "$resolved_worker_port" ]]; then
    AI_WORKER_PORT="$resolved_worker_port"
elif [[ -z "${AI_WORKER_PORT:-}" ]]; then
    echo "error: could not parse worker_port from supervisor reload response and AI_WORKER_PORT is unset" >&2
    exit 1
fi

echo "agent: using worker_port=${AI_WORKER_PORT} (from supervisor reload)"

AI_WORKER_PORT="$AI_WORKER_PORT" \
SUPERVISOR_PORT="$SUPERVISOR_PORT" \
PROJECT_DIR="${PROJECT_DIR:-/workspace/ai_sandbox/canon-mini-agent/prototype/ai}" \
CANON_OPENAI_BASE_URL="${CANON_OPENAI_BASE_URL:-http://127.0.0.1:8082/v1}" \
CANON_BROWSER_CDP_URL="${CANON_BROWSER_CDP_URL:-http://127.0.0.1:9223}" \
MCP_CONNECTOR_URL="${MCP_CONNECTOR_URL:-http://127.0.0.1:4000}" \
./target/release/agent
