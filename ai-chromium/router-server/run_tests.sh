#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

node src/tools/check-syntax.mjs
node --test --test-reporter=dot \
  test/openai-contract.test.mjs \
  test/mock-cdp-integration.test.mjs \
  test/artifact-quality.test.mjs

if [[ "${RUN_LIVE_TESTS:-0}" == "1" || -n "${LIVE_ROUTER_URL:-}" ]]; then
  node --test --test-reporter=dot test/live-cdp-9221.test.mjs
fi
