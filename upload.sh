git add .
git commit -m "uploading to chatgpt projects"
git push origin main
git rev-parse HEAD

python3 /workspace/ai_sandbox/canon-mini-agent/prototype/cdp-file-upload/upload_via_cdp.py \
  --build-tar \
  --tar-script /workspace/ai_sandbox/canon-mini-agent/prototype/ai/tar.sh \
  --tar-output ai.tar.gz \
  --open-target-if-missing \
  --target-url "https://chatgpt.com/g/g-p-69eedbc6bd38819180b138ab3c47abff-ai-prototype/project?tab=sources" \
  --match "chatgpt.com/g/g-p-69eedbc6bd38819180b138ab3c47abff-ai-prototype/project?tab=sources" \
  --target-wait-timeout-sec 45 \
  --open-sources-flow \
  --scope sources \
  --force-upload \
  --confirm-loaded \
  --confirm-timeout-sec 120 \
  --confirm-settle-sec 3 \
  "$@"
