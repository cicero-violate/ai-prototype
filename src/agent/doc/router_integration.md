# Router Integration

This file describes how the future Rust agent relates to the router-server and
what the agent can and cannot observe when ChatGPT uses MCP tools.

## Existing External Paths

```
router-server:   http://127.0.0.1:8081   (POST /v1/chat/completions)
mcp supervisor:  http://0.0.0.0:4000/mcp
mcp worker:      http://127.0.0.1:<port>/mcp_worker
```

## The Two Tool Execution Paths

These are completely separate. Conflating them is wrong.

### Path A — Rust calls MCP directly

```
Rust agent
  → LiveMcpCallExecutor.execute_call("shell", args)
  → POST http://127.0.0.1:<port>/mcp_worker  (JSON-RPC tools/call)
  → McpCallReceipt { response_hash, exit_status, receipt_hash }
  → EvidenceSubmission → kernel → TLog
```

The Rust agent knows exactly what tool was called, with what arguments, and what
the output was. A receipt is produced. The kernel sees the effect.

### Path B — ChatGPT calls MCP through the browser

```
Rust agent
  → POST router-server /v1/chat/completions  { messages, browser }
  → router-server → CDP → ChatGPT tab
  → ChatGPT decides to call an MCP tool
  → auto-approve loop approves the browser dialog
  → chatgpt-mcp-connector executes the tool
  → result flows back into ChatGPT's response stream
  → ChatGPT produces final text response
  → router-server returns: { content, browser.target_url, ... }
  → Rust agent receives: OpenAiChatResponse { content, target_url }
```

The Rust agent receives **only the final text content** of the ChatGPT response.
It does NOT receive:
- Which tools ChatGPT called
- What arguments were used
- What the tool outputs were
- Any receipt for those tool executions

The MCP tool calls in Path B are internal to the ChatGPT browser session.
They are invisible to the Rust side. No `McpCallReceipt` is produced.
The only evidence of the whole turn is the `OpenAiLlmEffectReceipt`.

## What `target_url` Is For

The router-server response includes `browser.target_url` — the ChatGPT tab URL
that served the turn. Pass this to the next turn via `OpenAiBrowserOptions::continue_at`
to maintain the same ChatGPT conversation thread across multiple turns.

Without it, each turn may open a new tab, losing context.

`OpenAiChatResponse::target_url` carries this value. The agent loop must thread
it between turns.

When the agent intentionally cleans up a tab, it uses browser-router's public tab
API on the configured `CANON_OPENAI_BASE_URL`: `GET /tabs` to resolve the target
ID from `target_url`, then `DELETE /tabs/{target_id}` to close it. The agent
must not call Chromium's `/json/close` endpoint directly.

## Relationship To Future Rust Agent

The Rust agent treats the router-server as an LLM transport. It does not treat it
as a tool execution surface.

```
router-server gives: model output (text) + target_url
capability records turn that output into: OpenAiLlmEffectReceipt
runtime verifies: the LLM gate transition
eval decides: quality of the response
learning promotes: only verified wins
```

MCP tool effects visible to the Rust side (Path A) go through `McpCallReceipt`.
MCP tool effects that happen inside ChatGPT (Path B) are not separately receipted.
If the agent needs to know what tools ChatGPT used, it must parse the response
content — which is unstructured text and not a reliable evidence source.

## Integration Shape

```
external agent loop (or future Rust agent)
  → router-server  (LLM turn, produces OpenAiLlmEffectReceipt)
  → ai worker command API  (submit evidence, advance kernel state)

separately, when the Rust agent itself wants to call tools:
  → LiveMcpCallExecutor  (tool turn, produces McpCallReceipt)
  → ai worker command API  (submit evidence, advance kernel state)
```

## What Needs To Be Defined

- [ ] How the agent decides whether to call a tool directly (Path A) vs ask ChatGPT to do it (Path B).
- [ ] Whether ChatGPT-driven tool calls need to be audited via response content parsing.
- [ ] How incomplete turns (ChatGPT timed out, tab lost) are handled and retried.
- [ ] How the agent threads `target_url` across a multi-turn conversation.
- [ ] What `OpenAiLlmEffectReceipt` fields are required for a turn to be considered receipted.
