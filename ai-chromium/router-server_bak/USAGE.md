# Browser LLM Router Server (CDP)

This server exposes an OpenAI-compatible API that drives existing browser tabs through Chrome DevTools Protocol (CDP).

```text
client → POST /v1/chat/completions → router-server → CDP websocket → ChatGPT/Gemini tab
```

## Run

Default (CDP on `9221`):

```bash
node src/server.mjs
```

Explicit CDP port (recommended when testing multiple Chrome instances):

```bash
CDP_PORT=9221 node src/server.mjs
# or
CDP_PORT=9222 node src/server.mjs
```

Defaults:

```text
HTTP API: http://127.0.0.1:8081
CDP:      http://127.0.0.1:9221
```

## Health

```bash
curl -sS http://127.0.0.1:8081/healthz
```

Health includes active providers and confirms which CDP endpoint the server is using.

## Models / Providers

Use one of:

- `chatgpt-group` → `chatgpt_group`
- `chatgpt-project` → `chatgpt_project`
- `chatgpt-browser` (or `chatgpt-cdp`) → `chatgpt_private`
- `gemini-browser` (or names containing `gemini`) → `gemini_private`

You can also force provider explicitly:

```json
{
  "browser": {
    "provider": "chatgpt_group"
  }
}
```

## Basic non-streaming request

```bash
curl -sS http://127.0.0.1:8081/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{
    "model": "chatgpt-browser",
    "messages": [{"role":"user","content":"Say hello in one sentence."}],
    "stream": false
  }'
```

## Streaming request

```bash
curl -N http://127.0.0.1:8081/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{
    "model": "chatgpt-browser",
    "messages": [{"role":"user","content":"Count to five."}],
    "stream": true
}'
```

## Group chat: create a new group thread

```bash
curl -sS -X POST http://127.0.0.1:8081/actions/group-chat \
  -H 'content-type: application/json' \
  -d '{}'
```

Behavior:

1. Opens `https://chatgpt.com/`
2. Clicks the group-chat start flow
3. Waits for navigation to a newly created chat URL
4. Closes the post-create popup (e.g. "Copy link")
5. Returns `group_chat_url`

## Group chat: send in a specific group thread

```bash
curl -sS http://127.0.0.1:8081/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{
    "model": "chatgpt-group",
    "messages": [{"role":"user","content":"Reply with OK only."}],
    "stream": false,
    "browser": {
      "provider": "chatgpt_group",
      "target_url": "https://chatgpt.com/gg/<group-id>",
      "create_group_chat": false,
      "reset_chat": false,
      "idle_ms": 5000,
      "max_ms": 120000
    }
  }'
```

## Stateful group-chat test (same thread, two turns)

Turn 1:

```bash
curl -sS http://127.0.0.1:8081/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{
    "model": "chatgpt-group",
    "messages": [{"role":"user","content":"Remember token BLUE-1234. Reply ACK."}],
    "stream": false,
    "browser": {
      "provider": "chatgpt_group",
      "target_url": "https://chatgpt.com/gg/<group-id>",
      "create_group_chat": false,
      "reset_chat": false
    }
}'
```

Turn 2 (same `target_url`):

```bash
curl -sS http://127.0.0.1:8081/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{
    "model": "chatgpt-group",
    "messages": [{"role":"user","content":"What token did I ask you to remember? Reply with token only."}],
    "stream": false,
    "browser": {
      "provider": "chatgpt_group",
      "target_url": "https://chatgpt.com/gg/<group-id>",
      "create_group_chat": false,
      "reset_chat": false
    }
  }'
```

## Request controls

```json
{
  "browser": {
    "provider": "chatgpt_group",
    "target_url": "https://chatgpt.com/gg/<group-id>",
    "reset_chat": false,
    "create_group_chat": false,
    "idle_ms": 2500,
    "first_capture_ms": 45000,
    "max_ms": 120000
  }
}
```

- `provider`: force adapter selection.
- `target_url`: send to a specific chat URL.
- `reset_chat`: create a fresh browser target/tab before running.
- `create_group_chat`: for `chatgpt_group`, force create flow from homepage.
- `idle_ms`: finalize after inactivity.
- `first_capture_ms`: fail/stop if no early response capture.
- `max_ms`: hard timeout.

## Notes

- If a specific tab is unstable (CDP websocket closes), use `reset_chat: true` or switch to a different CDP instance (`9221` vs `9222`).
- For reliable group-chat sends, pass explicit `browser.target_url` and keep it constant across turns for stateful behavior.

curl -sS http://127.0.0.1:8081/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{
    "model": "chatgpt-group",
    "messages": [{"role":"user","content":"Hello group chat"}],
    "stream": false,
    "browser": {
      "provider": "chatgpt_group",
      "target_url": "https://chatgpt.com/gg/<group-id>",
      "create_group_chat": false,
      "reset_chat": false
   }
}'

curl -sS http://127.0.0.1:8081/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{
    "model":"chatgpt-group",
    "messages":[{"role":"user","content":"Hello group chat"}],
    "stream":false,
    "browser":{
      "provider":"chatgpt_group",
      "create_group_chat":true,
      "reset_chat":false
   }
}'
