#!/usr/bin/env bash
set -euo pipefail

MODEL="${MODEL:-qwen2.5-coder:3b}"
HOST="${OLLAMA_HOST:-http://127.0.0.1:11434}"
MAX_STEPS="${MAX_STEPS:-8}"
TIMEOUT="${TIMEOUT:-30}"
AUTO_APPROVE="${AUTO_APPROVE:-0}"
GOAL="${*:-Inspect current directory and explain what you see.}"

WORKDIR="$(pwd)"
HISTORY="Goal: $GOAL"$'\n'
RAN_COMMAND=0

ask_model() {
python - "$HOST" "$MODEL" "$HISTORY" <<'PY'
import json, sys, urllib.request

host, model, history = sys.argv[1:]

system = """You are a local bash agent.
Return ONLY one-line valid JSON. No markdown. No prose outside JSON.
Schema:
{"done":false,"cmd":"bash command","reason":"short reason"}
When finished:
{"done":true,"cmd":"","reason":"final answer"}

Rules:
- You MUST issue at least one command before done=true.
- If asked what is in this directory, use: pwd && ls -la
- Prefer read-only commands first.
- Do not use sudo.
- Do not delete files.
- Do not install packages.
- Do not modify files unless explicitly asked.
- Keep commands short.
"""

payload = {
    "model": model,
    "messages": [
        {"role": "system", "content": system},
        {"role": "user", "content": history},
    ],
    "stream": False,
    "format": "json",
}

req = urllib.request.Request(
    host.rstrip("/") + "/v1/chat/completions",
    data=json.dumps(payload).encode(),
    headers={"content-type": "application/json"},
)

with urllib.request.urlopen(req) as r:
    data = json.load(r)

text = data["choices"][0]["message"]["content"].strip()

try:
    obj = json.loads(text)
except Exception:
    start = text.find("{")
    end = text.rfind("}") + 1
    if start < 0 or end <= start:
        obj = {"done": False, "cmd": "pwd && ls -la", "reason": "fallback after invalid JSON"}
    else:
        obj = json.loads(text[start:end])

print(json.dumps({
    "done": bool(obj.get("done", False)),
    "cmd": str(obj.get("cmd", "")),
    "reason": str(obj.get("reason", "")),
}))
PY
}

json_field() {
  python -c "import sys,json; print(json.load(sys.stdin).get('$1', ''))"
}

deny_cmd() {
  local cmd="$1"
  case "$cmd" in
    *"sudo "*|sudo*|*" rm -rf "*|*"rm -rf /"*|*"mkfs"*|*"dd if="*|*"shutdown"*|*"reboot"*|*"poweroff"*|*"chmod -R /"*|*"chown -R /"*)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

for step in $(seq 1 "$MAX_STEPS"); do
  JSON="$(ask_model)"

  DONE="$(json_field done <<<"$JSON")"
  CMD="$(json_field cmd <<<"$JSON")"
  REASON="$(json_field reason <<<"$JSON")"

  if [[ "$RAN_COMMAND" == "0" ]]; then
    DONE=false
    [[ -n "$CMD" ]] || CMD="pwd && ls -la"
    REASON="forced first command before done"
  fi

  echo
  echo "STEP $step"
  echo "REASON: $REASON"

  if [[ "$DONE" == "True" || "$DONE" == "true" ]]; then
    echo "$REASON"
    exit 0
  fi

  if [[ -z "$CMD" ]]; then
    echo "Empty command. Stopping."
    exit 1
  fi

  if deny_cmd "$CMD"; then
    echo "Denied dangerous command: $CMD"
    exit 1
  fi

  echo "CMD: $CMD"

  if [[ "$AUTO_APPROVE" != "1" ]]; then
    read -r -p "Run command? [y/N] " OK
    [[ "$OK" == "y" || "$OK" == "Y" ]] || exit 1
  fi

  set +e
  OUT="$(timeout "$TIMEOUT" bash -lc "cd '$WORKDIR' && $CMD" 2>&1)"
  CODE=$?
  set -e

  echo "$OUT"

  RAN_COMMAND=1
  HISTORY+=$'\n'"Step $step command: $CMD"$'\n'
  HISTORY+="Exit code: $CODE"$'\n'
  HISTORY+="Output:"$'\n'"$OUT"$'\n'
done

echo "Max steps reached."
