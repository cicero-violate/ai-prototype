#!/usr/bin/env bash
# Start the supervisor. It manages the TLog worker lifecycle and accepts
# POST /spawn requests to launch new agent threads inside the same process.
#
# Routes:
#   GET  :9100/health   — worker health + active port
#   POST :9100/reload   — hot-reload the worker
#   POST :9100/spawn    — spawn a new agent thread (domain, metric, max_steps)
#
# Usage: ./run_supervisor.sh
# Override any variable inline: SUPERVISOR_PORT=9200 ./run_supervisor.sh

SUPERVISOR_PORT="${SUPERVISOR_PORT:-9100}" \
PROJECT_DIR="${PROJECT_DIR:-/workspace/ai_sandbox/canon-mini-agent/prototype/ai}" \
CANON_OPENAI_BASE_URL="${CANON_OPENAI_BASE_URL:-http://127.0.0.1:8081/v1}" \
MCP_CONNECTOR_URL="${MCP_CONNECTOR_URL:-http://127.0.0.1:4000}" \
AI_TLOG_DIR="${AI_TLOG_DIR:-tlog}" \
./target/release/supervisor
