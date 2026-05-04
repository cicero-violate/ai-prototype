#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

node src/tools/check-syntax.mjs
node --test --test-force-exit --test-reporter=dot \
  test/openai-contract.test.mjs \
  test/mock-cdp-integration.test.mjs \
  test/openai-sdk-contract.test.mjs \
  test/artifact-quality.test.mjs \
  test/current-artifact-corpus.test.mjs \
  test/validation-profile.test.mjs

if [[ "${RUN_LIVE_TESTS:-0}" == "1" || -n "${LIVE_ROUTER_URL:-}" ]]; then
  node --test --test-force-exit --test-reporter=dot test/live-cdp-9221.test.mjs
fi
