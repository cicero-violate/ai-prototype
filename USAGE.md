# Usage

## Binaries

| Binary       | Role                                                              |
|--------------+-------------------------------------------------------------------|
| `supervisor` | HTTP server that manages the TLog worker and spawns agent threads |
| `worker`     | TLog HTTP server — records evidence, owns runtime state           |
| `agent`      | AI loop driver — reads GOAL.md, calls MCP tools, submits evidence |

All three are built with:

```sh
cargo build --release
```

---

## Standard startup (3 terminals)

### Terminal 1 — MCP connector

```sh
cd ../chatgpt-mcp-connector
MCP_WORKSPACE_ROOT=/workspace/ai_sandbox/canon-mini-agent/prototype \
cargo run
```

Serves MCP tools on `:4000`. Workspace root defaults to the parent of its CWD if `MCP_WORKSPACE_ROOT` is not set.

### Terminal 2 — Router server

```sh
cd ../router-server   # or chatgpt-agent-loop
npm run dev
```

CDP proxy that routes LLM calls to ChatGPT on `:8081`.

### Terminal 3 — Supervisor

```sh
./run_supervisor.sh
```

Or explicitly:

```sh
SUPERVISOR_PORT=9100 \
PROJECT_DIR=/workspace/ai_sandbox/canon-mini-agent/prototype/ai \
CANON_OPENAI_BASE_URL=http://127.0.0.1:8081/v1 \
MCP_CONNECTOR_URL=http://127.0.0.1:4000 \
AI_TLOG_DIR=state/tlog \
./target/release/supervisor
```

The supervisor starts the TLog worker on a random port and prints it to stderr. Check it:

```sh
curl http://127.0.0.1:9100/health
# → {"ok":true,"generation":1,"worker_port":XXXXX}
```

### Terminal 4 — Agent

```sh
./run.sh
```

Or explicitly:

```sh
AI_WORKER_PORT=<worker_port_from_health> \
SUPERVISOR_PORT=9100 \
PROJECT_DIR=/workspace/ai_sandbox/canon-mini-agent/prototype/ai \
CANON_OPENAI_BASE_URL=http://127.0.0.1:8081/v1 \
MCP_CONNECTOR_URL=http://127.0.0.1:4000 \
./target/release/agent
```

---

## Agent modes

### Loop mode (default)

Activates when `$PROJECT_DIR/GOAL.md` exists. The agent runs an infinite cycle:

1. POSTs to `$MCP_CONNECTOR_URL/workspace` to sync the project root.
2. Reads `GOAL.md` → sends a planning turn → LLM creates/updates `plan.md` and `score.md`.
3. Runs `EXECUTE_TURNS` execute turns → LLM implements the next step, runs checks, commits.
4. If `AI_WORKER_PORT` is set, runs a certification pass stamping evidence to the TLog.
5. Sleeps `LOOP_SLEEP_MS` ms, then repeats.

### Single-cycle mode

Fallback when no `GOAL.md` is present. Requires a running worker.

```sh
AI_WORKER_PORT=9091 \
AI_AGENT_DOMAIN="my task" \
AI_AGENT_METRIC="task complete" \
AI_AGENT_MAX_STEPS=20 \
./target/release/agent
```

---

## Spawning child agents

From within a running agent, ChatGPT can call the `canon_spawn_agent` MCP tool to launch a new agent thread inside the supervisor process. All threads share the same TLog (one worker, one evidence chain).

```json
{
  "name": "canon_spawn_agent",
  "arguments": {
    "domain": "implement src/domain/records.rs",
    "metric": "cargo test passes",
    "max_steps": 20
  }
}
```

`supervisor_port` defaults to `9100` — only set it if the supervisor is on a different port.

The supervisor handles the rest: it knows the active worker port, project dir, and MCP connector URL.

Spawn receipt:

```json
{
  "ok": true,
  "spawn_id": "agent-0",
  "pid": 12345,
  "domain": "implement src/domain/records.rs",
  "metric": "cargo test passes",
  "worker_port": 44321
}
```

`pid` is the supervisor's own process ID — all agent threads run inside it.

---

## Inter-agent messaging

Agents communicate via an append-only NDJSON mailbox at:

```
$PROJECT_DIR/agent_state/mailbox/{agent_id}.ndjson
```

### Send a message

```json
{
  "name": "canon_send_agent_message",
  "arguments": {
    "sender": "agent-0",
    "target_agent": "agent-1",
    "message_kind": "TaskAssignment",
    "payload": "{\"task\": \"implement records.rs\"}"
  }
}
```

### Read the mailbox

```json
{
  "name": "canon_read_mailbox",
  "arguments": {
    "agent_id": "agent-1",
    "since": 0
  }
}
```

Returns `{messages: [...], next_cursor: N}`. Pass `next_cursor` as `since` on the next call to read only new messages.

Message kinds: `DomainSignal`, `GraphEditRequest`, `EvalRequest`, `PolicyCandidate`, `Observation`, `TaskAssignment`.

---

## Supervisor routes

```
GET  :9100/health   — {"ok":true,"generation":N,"worker_port":XXXXX}
POST :9100/reload   — hot-reload the worker (fresh TLog state)
POST :9100/spawn    — {"domain":"...","metric":"...","max_steps":20}
```

---

## Environment reference

### Agent

```text
PROJECT_DIR              default: derived from binary path (ai/ project root)
EXECUTE_TURNS            execute turns per cycle (default 5)
TURN_RETRY_LIMIT         retries per incomplete turn (default 2)
LOOP_SLEEP_MS            sleep between cycles in ms (default 5000)
AGENT_COUNT              parallel agent threads with 5s staggered starts (default 1)
MCP_CONNECTOR_URL        default http://127.0.0.1:4000
SSE_CHUNKS_DIR           default $PROJECT_DIR/agent_state/sse-chunks
ROUTER_TURN_MAX_MS       max ms waiting for LLM (default 600000)
ROUTER_FIRST_CAPTURE_MS  ms until first capture (default 60000)
ROUTER_IDLE_MS           idle threshold before capture completes (default 2500)
CANON_OPENAI_BASE_URL    router-server base URL (default http://127.0.0.1:8081/v1)
AI_WORKER_PORT           enables TLog certification when set
SUPERVISOR_PORT          when set, POST /reload before each certification
AI_CERT_MAX_STEPS        AgentCycle step budget for certification (default 30)
```

### Supervisor

```text
SUPERVISOR_PORT          HTTP port (default 9100)
AI_TLOG_DIR              TLog directory (default state/tlog)
AI_WORKER_BIN            worker binary path (default: sibling of supervisor binary)
AI_MCP_WORKER_URL        MCP worker URL (default http://127.0.0.1:38469/mcp_worker)
PROJECT_DIR              passed to spawned agent threads
CANON_OPENAI_BASE_URL    passed to spawned agent threads via process env
MCP_CONNECTOR_URL        passed to spawned agent threads
```

### Worker (standalone)

```text
PORT          HTTP port to listen on
AI_TLOG_DIR   TLog directory
```
