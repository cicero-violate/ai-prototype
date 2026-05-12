#!/usr/bin/env bash
set -euo pipefail

# Reload the supervisor only after a hook validation script has already passed.
#
# Defaults are conservative:
# - missing supervisor is informational so local commits do not fail just
#   because the supervisor is not running;
# - set REQUIRE_SUPERVISOR_RELOAD=1 to make reload failure fatal in controlled
#   automation.

SUPERVISOR_PORT="${SUPERVISOR_PORT:-9100}"
REQUIRE_SUPERVISOR_RELOAD="${REQUIRE_SUPERVISOR_RELOAD:-0}"
URL="http://127.0.0.1:${SUPERVISOR_PORT}/reload"

if command -v curl >/dev/null 2>&1; then
  if curl --fail --silent --show-error --max-time 5 -X POST "$URL" >/dev/null; then
    echo "supervisor reload: pass port=${SUPERVISOR_PORT}"
    exit 0
  fi
else
  python3 - "$URL" <<'PY'
import sys
import urllib.error
import urllib.request

url = sys.argv[1]
try:
    request = urllib.request.Request(url, data=b"", method="POST")
    with urllib.request.urlopen(request, timeout=5) as response:
        if 200 <= response.status < 300:
            print("supervisor reload: pass")
            raise SystemExit(0)
        print(f"supervisor reload: http {response.status}", file=sys.stderr)
except (OSError, urllib.error.URLError, urllib.error.HTTPError) as exc:
    print(f"supervisor reload: {exc}", file=sys.stderr)
raise SystemExit(1)
PY
  if [ "$?" -eq 0 ]; then
    exit 0
  fi
fi

if [ "$REQUIRE_SUPERVISOR_RELOAD" = "1" ]; then
  echo "supervisor reload: failed and REQUIRE_SUPERVISOR_RELOAD=1" >&2
  exit 1
fi

echo "supervisor reload: skipped or unavailable on port=${SUPERVISOR_PORT}" >&2
exit 0