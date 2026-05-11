#!/usr/bin/env bash
# Run the agent in loop mode against the running supervisor + MCP connector.
# The supervisor must already be running (see run_supervisor.sh).
#
# Usage: ./run.sh
# Override any variable inline: AI_WORKER_PORT=9091 ./run.sh

AI_WORKER_PORT="${AI_WORKER_PORT:-9091}" \
SUPERVISOR_PORT="${SUPERVISOR_PORT:-9100}" \
PROJECT_DIR="${PROJECT_DIR:-/workspace/ai_sandbox/canon-mini-agent/prototype/ai}" \
CANON_OPENAI_BASE_URL="${CANON_OPENAI_BASE_URL:-http://127.0.0.1:8081/v1}" \
MCP_CONNECTOR_URL="${MCP_CONNECTOR_URL:-http://127.0.0.1:4000}" \
./target/release/agent
