#!/usr/bin/env bash
set -euo pipefail

python3 "${CDP_UPLOAD_SCRIPT:-/mnt/data/canon-mini-agent-extracted/canon-mini-agent/prototype/cdp-file-upload/upload_via_cdp.py}" \
  --build-tar \
  --tar-script "${TAR_SCRIPT:-/mnt/data/canon-mini-agent-extracted/canon-mini-agent/prototype/ai/ai-chromium/tar.sh}" \
  --tar-output "${TAR_OUTPUT:-router-server.tar.gz}" \
  --cdp "${CDP_ENDPOINT:-http://127.0.0.1:9222}" \
  --open-target-if-missing \
  --target-url "${TARGET_URL:-https://chatgpt.com/g/g-p-69eedbc6bd38819180b138ab3c47abff-ai-prototype/project?tab=sources}" \
  --match "${TARGET_MATCH:-chatgpt.com/g/g-p-69eedbc6bd38819180b138ab3c47abff-ai-prototype/project?tab=sources}" \
  --target-wait-timeout-sec "${TARGET_WAIT_TIMEOUT_SEC:-45}" \
  --open-sources-flow \
  --scope sources \
  --force-upload \
  --confirm-loaded \
  --confirm-timeout-sec "${CONFIRM_TIMEOUT_SEC:-120}" \
  --confirm-settle-sec "${CONFIRM_SETTLE_SEC:-3}"
