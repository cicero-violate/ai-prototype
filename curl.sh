#!/usr/bin/env bash
# curl.sh — spawn a canon sub-agent and stream its evidence in real time.
#
# The spawned agent proves its Invariant gate against domain+metric bytes,
# runs execute-only turns (no separate planning turn), submits gate evidence
# to the shared kernel, closes its browser tab on exit, and feeds the
# policy-promotion learning loop.
#
# Usage:
#   ./curl.sh "<domain>" "<metric>" [max_steps]
#
# Examples:
#   ./curl.sh "fix flaky test in router_test.rs" "cargo test passes" 15
#   ./curl.sh "add exponential back-off to http client" "all tests green" 20
#   ./curl.sh "score current GOAL.md checkpoint" "SCORE_REPORT >= 80" 5
#
# Environment:
#   SUPERVISOR_PORT   default 9100
#   PROTOTYPE_DIR     default: parent of this script's directory
#   IDLE_SECS         seconds with no new tool calls before auto-exit (default 45)

set -euo pipefail

DOMAIN="${1:?Usage: $0 \"<domain>\" \"<metric>\" [max_steps]}"
METRIC="${2:?Usage: $0 \"<domain>\" \"<metric>\" [max_steps]}"
MAX_STEPS="${3:-20}"
SUPERVISOR_PORT="${SUPERVISOR_PORT:-9100}"
IDLE_SECS="${IDLE_SECS:-45}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROTOTYPE_DIR="${PROTOTYPE_DIR:-$(cd "$SCRIPT_DIR/.." && pwd)}"
SUPERVISOR_URL="http://127.0.0.1:${SUPERVISOR_PORT}"

EVIDENCE_FILE="${PROTOTYPE_DIR}/agent_state/tool_evidence.ndjson"
POLICY_FILE="${PROTOTYPE_DIR}/state/policy.ndjson"
SCORE_FILE="${PROTOTYPE_DIR}/SCORE_REPORT.md"

# ── helpers ────────────────────────────────────────────────────────────────────

die()    { echo "error: $*" >&2; exit 1; }
hr()     { printf '──────────────────────────────────────────────────────\n'; }
banner() { printf '┌──────────────────────────────────────────────────────┐\n'
           printf '│  %-52s│\n' "$*"
           printf '└──────────────────────────────────────────────────────┘\n'; }

# Print a batch of new evidence lines (one jq call for efficiency).
print_evidence_batch() {
    local from="$1" file="$2"
    tail -n "+${from}" "$file" 2>/dev/null \
    | jq -rR '
        . as $raw |
        try (
            (. | fromjson) as $r |
            (($r.ts // "?")[11:19]) + "  " +
            (if ($r.is_error // false) then "✗" else "✓" end) + "  " +
            (($r.tool // "?") | .[0:20] | . + (" " * (20 - length))) +
            (if (($r.gate // "") != "") then "  [\($r.gate)]" else "" end) +
            "  " + (($r.intent // "") | .[0:70])
        ) catch ("  <parse error>: " + $raw)
    ' 2>/dev/null || true
}

# Summarise gate coverage from new evidence lines (show pass/fail per gate).
gate_summary() {
    local from="$1" file="$2"
    tail -n "+${from}" "$file" 2>/dev/null \
    | jq -rs '
        [.[] | select(.gate? and .gate != "") |
         {gate: .gate, ok: (.is_error | not)}] |
        group_by(.gate) |
        map({gate: .[0].gate,
             pass: (map(select(.ok)) | length),
             fail: (map(select(.ok | not)) | length)}) |
        .[] | "  \(.gate): \(.pass)✓  \(.fail)✗"
    ' 2>/dev/null || true
}

# Latest policy version from ndjson store (max across all entries).
policy_version() {
    [ -f "$POLICY_FILE" ] || { echo 0; return; }
    jq -rs '[.[].version // 0] | max // 0' "$POLICY_FILE" 2>/dev/null || echo 0
}

# ── 1. supervisor health ───────────────────────────────────────────────────────

health=$(curl -sf "${SUPERVISOR_URL}/health" 2>/dev/null) \
    || die "supervisor not responding at ${SUPERVISOR_URL} — run run_supervisor.sh first"

worker_port=$(echo "$health" | jq -r '.worker_port')
generation=$(echo "$health" | jq -r '.generation')
echo "supervisor OK  generation=${generation}  worker_port=${worker_port}"

# ── 2. snapshot cursors before spawn ─────────────────────────────────────────

evidence_cursor=$(wc -l < "$EVIDENCE_FILE" 2>/dev/null || echo 0)
policy_v_before=$(policy_version)

# ── 3. spawn ───────────────────────────────────────────────────────────────────

spawn_body=$(jq -n \
    --arg     domain    "$DOMAIN" \
    --arg     metric    "$METRIC" \
    --argjson max_steps "$MAX_STEPS" \
    '{domain: $domain, metric: $metric, max_steps: $max_steps}')

spawn_resp=$(curl -sf -X POST "${SUPERVISOR_URL}/spawn" \
    -H 'Content-Type: application/json' \
    -d "$spawn_body") \
    || die "POST ${SUPERVISOR_URL}/spawn failed"

spawn_id=$(echo "$spawn_resp"  | jq -r '.spawn_id')
s_domain=$(echo "$spawn_resp"  | jq -r '.domain')
s_metric=$(echo "$spawn_resp"  | jq -r '.metric')
s_port=$(echo "$spawn_resp"    | jq -r '.worker_port')

printf '\n'
banner "spawned: ${spawn_id}"
printf '  domain:  %s\n' "$s_domain"
printf '  metric:  %s\n' "$s_metric"
printf '  steps:   %s\n' "$MAX_STEPS"
printf '  kernel:  port=%s\n' "$s_port"
printf '\n'

# ── 4. stream tool evidence ────────────────────────────────────────────────────

hr
echo "  tool evidence stream  (Ctrl-C to detach, agent keeps running)"
hr

idle=0
last_line=$((evidence_cursor + 1))   # tail -n +N is 1-based

while true; do
    current=$(wc -l < "$EVIDENCE_FILE" 2>/dev/null || echo 0)

    if [ "$current" -ge "$last_line" ]; then
        print_evidence_batch "$last_line" "$EVIDENCE_FILE"
        last_line=$((current + 1))
        idle=0
    else
        idle=$((idle + 3))
        if [ "$idle" -ge "$IDLE_SECS" ]; then
            printf '  (quiet for %ds — agent finished or paused)\n' "$IDLE_SECS"
            break
        fi
    fi

    sleep 3
done

# ── 5. kernel state ────────────────────────────────────────────────────────────

printf '\n'
hr
state=$(curl -sf "http://127.0.0.1:${s_port}/v1/state" 2>/dev/null || echo '{}')
phase=$(echo "$state"    | jq -r '.phase    // "unknown"')
tlog_len=$(echo "$state" | jq -r '.tlog_len // "?"')
failure=$(echo "$state"  | jq -r '.failure  // ""')
recovery=$(echo "$state" | jq -r '.recovery_action // ""')

printf '  phase=%s  tlog=%s' "$phase" "$tlog_len"
[ -n "$failure"  ] && printf '  failure=%s'  "$failure"
[ -n "$recovery" ] && printf '  recovery=%s' "$recovery"
printf '\n'
hr

# ── 6. gate coverage from this spawn ──────────────────────────────────────────

printf '\n  gate coverage (tool calls with gate field):\n'
gate_summary "$((evidence_cursor + 1))" "$EVIDENCE_FILE" \
    || printf '  (none recorded)\n'

# ── 7. policy promotion ────────────────────────────────────────────────────────

printf '\n'
hr
policy_v_after=$(policy_version)
if [ "$policy_v_after" -gt "$policy_v_before" ]; then
    printf '  policy promoted: v%s → v%s  (learning loop closed ✓)\n' \
        "$policy_v_before" "$policy_v_after"
    jq -r '"  latest: hash=\(.value // "?")  seq=\(.seq // "?")"' \
        "$POLICY_FILE" 2>/dev/null | tail -1 || true
else
    printf '  no policy promotion this spawn  (v%s unchanged)\n' "$policy_v_before"
    printf '  learning requires: Judgment + Eval gates both pass in same TLog cycle\n'
fi
hr

# ── 8. score report ────────────────────────────────────────────────────────────

printf '\n'
if [ -f "$SCORE_FILE" ]; then
    printf '  SCORE_REPORT.md:\n'
    sed 's/^/  /' "$SCORE_FILE" | head -20
else
    printf '  SCORE_REPORT.md: (none)\n'
fi

# ── 9. summary ─────────────────────────────────────────────────────────────────

printf '\n'
hr
new_calls=$(( $(wc -l < "$EVIDENCE_FILE" 2>/dev/null || echo 0) - evidence_cursor ))
printf '  spawn_id:   %s\n' "$spawn_id"
printf '  tool calls: %d new\n' "$new_calls"
printf '  phase:      %s\n' "$phase"
printf '  policy:     v%s\n' "$policy_v_after"
hr
printf '\n'
