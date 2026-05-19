curl -sS -X POST http://127.0.0.1:9100/spawn \
  -H "content-type: application/json" \
  -d '{"domain":"rust","metric":"Do ls -la","max_steps":2}'
