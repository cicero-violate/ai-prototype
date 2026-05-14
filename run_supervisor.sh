#!/usr/bin/env bash
set -euo pipefail

# Canon supervisor launcher.
#
# Single behavior, no connector-control modes:
#   - define/export all env used by supervisor, worker, agent, and MCP paths
#   - build ai dev + release
#   - start supervisor on SUPERVISOR_PORT, default 9100
#   - never start, stop, kill, or restart the MCP connector
#
# The MCP connector is a separate ChatGPT-facing service. Keep it running
# independently on MCP_CONNECTOR_URL, default http://127.0.0.1:4000.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROTOTYPE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
CONNECTOR_DIR_DEFAULT="$PROTOTYPE_DIR/chatgpt-mcp-connector"

url_port() {
  local url="$1"
  url="${url#http://}"
  url="${url#https://}"
  url="${url%%/*}"
  if [[ "$url" == *:* ]]; then
    printf '%s\n' "${url##*:}"
  else
    printf '4000\n'
  fi
}

port_is_listening() {
  local port="$1"
  if command -v ss >/dev/null 2>&1; then
    ss -ltn 2>/dev/null | awk '{print $4}' | grep -Eq "(^|:)$port$"
  elif command -v netstat >/dev/null 2>&1; then
    netstat -ltn 2>/dev/null | awk '{print $4}' | grep -Eq "(^|:)$port$"
  else
    return 1
  fi
}

# Supervisor / AI env.
export SUPERVISOR_PORT="${SUPERVISOR_PORT:-9100}"
export PROJECT_DIR="${PROJECT_DIR:-$SCRIPT_DIR}"
export AI_TLOG_DIR="${AI_TLOG_DIR:-$PROTOTYPE_DIR/state/tlog}"
export AI_WORKER_BIN="${AI_WORKER_BIN:-$PROTOTYPE_DIR/target/release/worker}"
export AI_AGENT_BIN="${AI_AGENT_BIN:-$PROTOTYPE_DIR/target/release/agent}"
export AI_WORKER_PORT="${AI_WORKER_PORT:-$SUPERVISOR_PORT}"

# Router / agent env.
export CANON_OPENAI_BASE_URL="${CANON_OPENAI_BASE_URL:-http://127.0.0.1:8081/v1}"
export SSE_CHUNKS_DIR="${SSE_CHUNKS_DIR:-$PROJECT_DIR/agent_state/sse-chunks}"
export ROUTER_TURN_MAX_MS="${ROUTER_TURN_MAX_MS:-600000}"
export ROUTER_FIRST_CAPTURE_MS="${ROUTER_FIRST_CAPTURE_MS:-60000}"
export ROUTER_IDLE_MS="${ROUTER_IDLE_MS:-2500}"
export AI_AGENT_DOMAIN="${AI_AGENT_DOMAIN:-}"
export AI_AGENT_METRIC="${AI_AGENT_METRIC:-}"

# MCP connector env known to AI/supervisor/agent paths.
export MCP_CONNECTOR_URL="${MCP_CONNECTOR_URL:-http://127.0.0.1:4000}"
export MCP_CONNECTOR_PORT="${MCP_CONNECTOR_PORT:-$(url_port "$MCP_CONNECTOR_URL")}"
export MCP_CONNECTOR_DIR="${MCP_CONNECTOR_DIR:-$CONNECTOR_DIR_DEFAULT}"
export MCP_WORKSPACE_ROOT="${MCP_WORKSPACE_ROOT:-$PROTOTYPE_DIR}"
export MCP_ALLOWED_WORKSPACE_ROOT="${MCP_ALLOWED_WORKSPACE_ROOT:-$PROTOTYPE_DIR}"
export MCP_WORKER_BIN="${MCP_WORKER_BIN:-$PROTOTYPE_DIR/target/release/chatgpt-mcp-connector}"
export MCP_WORKER_RESPONSE_TIMEOUT_SECS="${MCP_WORKER_RESPONSE_TIMEOUT_SECS:-600}"
export AI_MCP_WORKER_URL="${AI_MCP_WORKER_URL:-http://127.0.0.1:38469/mcp_worker}"
export BASE_URL="${BASE_URL:-https://cheese-server.duckdns.org}"
export APPLY_PATCH_BIN="${APPLY_PATCH_BIN:-apply_patch}"

print_effective_env() {
  cat >&2 <<EOF
run_supervisor: effective env
  SUPERVISOR_PORT=$SUPERVISOR_PORT
  AI_WORKER_PORT=$AI_WORKER_PORT
  AI_WORKER_BIN=$AI_WORKER_BIN
  AI_AGENT_BIN=$AI_AGENT_BIN
  AI_TLOG_DIR=$AI_TLOG_DIR
  PROJECT_DIR=$PROJECT_DIR
  CANON_OPENAI_BASE_URL=$CANON_OPENAI_BASE_URL
  MCP_CONNECTOR_URL=$MCP_CONNECTOR_URL
  MCP_CONNECTOR_PORT=$MCP_CONNECTOR_PORT
  AI_MCP_WORKER_URL=$AI_MCP_WORKER_URL
  SSE_CHUNKS_DIR=$SSE_CHUNKS_DIR
  ROUTER_TURN_MAX_MS=$ROUTER_TURN_MAX_MS
  ROUTER_FIRST_CAPTURE_MS=$ROUTER_FIRST_CAPTURE_MS
  ROUTER_IDLE_MS=$ROUTER_IDLE_MS
  AI_AGENT_DOMAIN=$AI_AGENT_DOMAIN
  AI_AGENT_METRIC=$AI_AGENT_METRIC
  MCP_CONNECTOR_DIR=$MCP_CONNECTOR_DIR
  MCP_WORKSPACE_ROOT=$MCP_WORKSPACE_ROOT
  MCP_ALLOWED_WORKSPACE_ROOT=$MCP_ALLOWED_WORKSPACE_ROOT
  MCP_WORKER_BIN=$MCP_WORKER_BIN
  MCP_WORKER_RESPONSE_TIMEOUT_SECS=$MCP_WORKER_RESPONSE_TIMEOUT_SECS
  BASE_URL=$BASE_URL
  APPLY_PATCH_BIN=$APPLY_PATCH_BIN
EOF
}

cd "$PROTOTYPE_DIR"
mkdir -p "$PROTOTYPE_DIR/state/rustc" "$SSE_CHUNKS_DIR"

print_effective_env

if port_is_listening "$MCP_CONNECTOR_PORT"; then
  echo "run_supervisor: MCP connector is listening on port $MCP_CONNECTOR_PORT; leaving it untouched" >&2
else
  echo "run_supervisor: MCP connector is not listening on port $MCP_CONNECTOR_PORT; leaving it untouched" >&2
fi

cargo build
cargo build --release

SUPERVISOR_PORT="$SUPERVISOR_PORT" \
PROJECT_DIR="$PROJECT_DIR" \
CANON_OPENAI_BASE_URL="$CANON_OPENAI_BASE_URL" \
MCP_CONNECTOR_URL="$MCP_CONNECTOR_URL" \
AI_TLOG_DIR="$AI_TLOG_DIR" \
AI_WORKER_BIN="$AI_WORKER_BIN" \
AI_AGENT_BIN="$AI_AGENT_BIN" \
AI_MCP_WORKER_URL="$AI_MCP_WORKER_URL" \
AI_WORKER_PORT="$AI_WORKER_PORT" \
SSE_CHUNKS_DIR="$SSE_CHUNKS_DIR" \
ROUTER_TURN_MAX_MS="$ROUTER_TURN_MAX_MS" \
ROUTER_FIRST_CAPTURE_MS="$ROUTER_FIRST_CAPTURE_MS" \
ROUTER_IDLE_MS="$ROUTER_IDLE_MS" \
AI_AGENT_DOMAIN="$AI_AGENT_DOMAIN" \
AI_AGENT_METRIC="$AI_AGENT_METRIC" \
"$PROTOTYPE_DIR/target/release/supervisor"