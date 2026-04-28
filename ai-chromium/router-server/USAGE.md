# Browser LLM Router Server

This server turns the Chromium extension into an OpenAI-compatible local API.

```text
client → POST /v1/chat/completions → local server → ws://127.0.0.1:9100 → extension → ChatGPT/Gemini tab
```

## Run

```bash
node server/browser-llm-router.mjs
```

Defaults:

```text
HTTP API:  http://127.0.0.1:8080
Extension: ws://127.0.0.1:9100
```

The existing `background.js` already connects to `ws://127.0.0.1:9100`, so no extension code change is required.

## Health

```bash
curl http://127.0.0.1:8080/healthz
```

## Non-streaming request

```bash
curl http://127.0.0.1:8080/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{
    "model": "chatgpt-browser",
    "messages": [{"role":"user","content":"Say hello in one sentence."}],
    "stream": false
  }'
```

## Streaming request

```bash
curl -N http://127.0.0.1:8080/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{
    "model": "chatgpt-browser",
    "messages": [{"role":"user","content":"Count to five."}],
    "stream": true
  }'
```

## Gemini route

```bash
curl -N http://127.0.0.1:8080/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{
    "model": "gemini-browser",
    "messages": [{"role":"user","content":"Say hello."}],
    "stream": true
  }'
```

## Request controls

Optional request fields:

```json
{
  "browser": {
    "url": "https://chatgpt.com/",
    "reset_chat": true,
    "close_tab": false,
    "idle_ms": 2500,
    "max_ms": 120000,
    "raw_fallback": true
  }
}
```

- `reset_chat`: starts a new chat before each request.
- `idle_ms`: ends the API response after this much silence from the extension.
- `max_ms`: hard maximum turn time.
- `raw_fallback`: returns raw captured chunks when clean text extraction fails.

## Notes

The extension emits streamed chunks but does not emit a final `TURN_DONE` event. The server therefore finalizes a response using an idle timeout after the last inbound chunk.
