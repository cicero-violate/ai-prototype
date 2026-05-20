#!/usr/bin/env bash
# Launch the canon agent loop via the supervisor API.
# The supervisor must already be running (cargo run --release).
#
# Usage:
#   ./run_agent_loop.sh [options]
#
# Options:
#   --supervisor-url URL    Supervisor base URL                    (default: http://127.0.0.1:9100)
#   --cwd PATH              Working/project directory              (default: parent of ai/ dir)
#   --execute-turns N       Execute turns per cycle                (default: supervisor env or 2)
#   --mini-agent-count N    DAG mini-agents per scheduling wave    (default: 3, max: 5)
#   --agent-count N         Parallel agents, 1-5                   (default: env AGENT_COUNT or 1)
#   --wait                  Poll until supervisor is ready first
#   -h, --help              Show this help

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

SUPERVISOR_URL="${SUPERVISOR_URL:-http://127.0.0.1:9100}"
CWD="${PROJECT_DIR:-${SCRIPT_DIR%/ai}}"
EXECUTE_TURNS=""
MINI_AGENT_COUNT=""
MAX_AGENT_COUNT=5
AGENT_COUNT="${AGENT_COUNT:-1}"
WAIT=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --supervisor-url)   SUPERVISOR_URL="$2";   shift 2 ;;
    --cwd)              CWD="$2";              shift 2 ;;
    --execute-turns)    EXECUTE_TURNS="$2";    shift 2 ;;
    --mini-agent-count) MINI_AGENT_COUNT="$2"; shift 2 ;;
    --agent-count)      AGENT_COUNT="$2";      shift 2 ;;
    --wait)           WAIT=1;              shift   ;;
    -h|--help)
      awk '/^#/ { sub(/^# ?/, ""); print; next } { exit }' "$0"
      exit 0
      ;;
    *) echo "unknown option: $1" >&2; exit 1 ;;
  esac
done

if ! [[ "$AGENT_COUNT" =~ ^[0-9]+$ ]]; then
  echo "agent count must be a number from 1 to $MAX_AGENT_COUNT: $AGENT_COUNT" >&2
  exit 1
fi
if (( AGENT_COUNT < 1 || AGENT_COUNT > MAX_AGENT_COUNT )); then
  echo "agent count must be between 1 and $MAX_AGENT_COUNT: $AGENT_COUNT" >&2
  exit 1
fi

if [[ "$WAIT" -eq 1 ]]; then
  printf 'Waiting for supervisor at %s ...' "$SUPERVISOR_URL"
  until curl -fsS --max-time 2 "$SUPERVISOR_URL/health" >/dev/null 2>&1; do
    printf '.'
    sleep 1
  done
  echo " ready."
fi

# Build JSON body — always include bounded agent_count; include turns only when set.
BODY='{'
BODY+="\"working_dir\":\"${CWD}\""
[[ -n "$EXECUTE_TURNS" ]]    && BODY+=",\"execute_turns\":${EXECUTE_TURNS}"
[[ -n "$MINI_AGENT_COUNT" ]] && BODY+=",\"mini_agent_count\":${MINI_AGENT_COUNT}"
BODY+=",\"agent_count\":${AGENT_COUNT}"
BODY+='}'

echo "Starting agent loop:"
echo "  supervisor : $SUPERVISOR_URL"
echo "  cwd        : $CWD"
[[ -n "$EXECUTE_TURNS" ]]    && echo "  turns      : $EXECUTE_TURNS"
[[ -n "$MINI_AGENT_COUNT" ]] && echo "  mini-agents: $MINI_AGENT_COUNT"
echo "  agents     : $AGENT_COUNT"
echo ""

curl -fsSX POST "$SUPERVISOR_URL/agent/start" \
  -H "Content-Type: application/json" \
  -d "$BODY" | jq .
