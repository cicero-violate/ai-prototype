# Usage

The `agent` binary has two modes:

- **Loop mode** — reads `GOAL.md` from `PROJECT_DIR`, syncs the MCP workspace, then runs an infinite cycle of 1 planning turn + N execute turns against the router-server. No canon worker required. This mirrors `chatgpt-agent-loop`.
- **Single-cycle mode** — falls back to the phase-driven `AgentCycle` when no `GOAL.md` is found. The worker owns runtime state; the agent observes phase, asks the LLM for phase-appropriate work, and submits evidence gates itself.

## Build

```sh
cargo build --release
```

If the local `canon-rustc-v3` wrapper fails to load `librustc_driver`, disable it for the build:

```sh
cargo build --release --config 'build.rustc-wrapper=""'
```

## Prerequisite: Router Server

Both modes require the router-server running before the first turn. Default endpoint:

```text
http://127.0.0.1:8081/v1/chat/completions
```

Override with:

```sh
export CANON_OPENAI_BASE_URL=http://127.0.0.1:8081/v1
```

---

## Loop Mode

Loop mode activates automatically when `$PROJECT_DIR/GOAL.md` exists.

### Minimal setup

Create a goal file:

```sh
mkdir -p /path/to/project
cat > /path/to/project/GOAL.md <<'EOF'
Implement the feature described below and add tests.
...
EOF
```

Run the agent:

```sh
PROJECT_DIR=/workspace/ai_sandbox/canon-mini-agent/prototype \
CANON_OPENAI_BASE_URL=http://127.0.0.1:8081/v1 \
./target/release/agent
```

Each cycle the agent:
1. Posts to `$MCP_CONNECTOR_URL/workspace` to sync the project root (fails fast if unreachable).
2. Reads `GOAL.md` and sends a planning turn — the LLM creates/updates `plan.md` and `score.md`.
3. Runs `EXECUTE_TURNS` execute turns — the LLM reads `plan.md`, implements the next step, runs checks, and commits.
4. Sleeps `LOOP_SLEEP_MS` ms before the next cycle.

Chunk logs are written to `$SSE_CHUNKS_DIR` (one ndjson file per turn attempt).

### Loop mode environment

```text
PROJECT_DIR              optional; default cwd; must contain GOAL.md
EXECUTE_TURNS            optional; execute turns per cycle (default 5)
TURN_RETRY_LIMIT         optional; retries per incomplete turn (default 2)
LOOP_SLEEP_MS            optional; sleep between turns and cycles in ms (default 5000)
AGENT_COUNT              optional; parallel agents with 5 s staggered starts (default 1)
MCP_CONNECTOR_URL        optional; default http://127.0.0.1:4000
SSE_CHUNKS_DIR           optional; default $PROJECT_DIR/agent_state/sse-chunks
ROUTER_TURN_MAX_MS       optional; max ms the router waits for the LLM (default 600000)
ROUTER_FIRST_CAPTURE_MS  optional; ms until first response capture (default 60000)
ROUTER_IDLE_MS           optional; idle threshold before capture completes (default 2500)
CANON_OPENAI_BASE_URL    optional; router-server base URL (default http://127.0.0.1:8081/v1)
CANON_OPENAI_MODEL       optional; model identifier forwarded to router-server
CANON_OPENAI_TIMEOUT_MS  optional; TCP read/write timeout for each turn
```

### Evidence certification (tlog / receipts)

When `AI_WORKER_PORT` is set, the agent runs an `AgentCycle` certification pass **after each successful loop cycle**. The cycle drives the worker through all evidence gates (Invariant → Analysis → Judgment → Plan → Execute → Verify → Eval → Done), stamping LLM receipts to the tlog.

The objective for the certification is built from the files the loop cycle just produced:
- `GOAL.md` → `domain_hint` (up to 800 chars)
- `plan.md` → appended to domain_hint as "Completed work" summary (up to 400 chars)
- `score.md` → `success_metric` (up to 300 chars)

The worker must be in a state where it can accept new evidence (i.e. not already at `phase=Done` from a prior run). If `SUPERVISOR_PORT` is also set, the agent calls `POST /reload` on the supervisor before each certification, waits up to 30 s for the worker to become healthy, then certifies against the fresh instance.

```sh
# Minimal: loop + certification against a running worker
AI_WORKER_PORT=9091 \
PROJECT_DIR=/path/to/project \
CANON_OPENAI_BASE_URL=http://127.0.0.1:8081/v1 \
./target/release/agent

# With supervisor auto-reload for a fresh worker each cycle
AI_WORKER_PORT=9091 \
SUPERVISOR_PORT=9090 \
PROJECT_DIR=/path/to/project \
CANON_OPENAI_BASE_URL=http://127.0.0.1:8081/v1 \
./target/release/agent
```

Certification environment:

```text
AI_WORKER_PORT      required to enable certification; worker HTTP port
SUPERVISOR_PORT     optional; when set, POST /reload is called before each cert
AI_CERT_MAX_STEPS   optional; AgentCycle step budget for certification (default 30)
```

Certification log lines are prefixed `[agent] cert:`:

```text
[agent] cert: cycle 1 — domain=Implement the feature…
[agent] cert: done  steps=9  success=true  stop=phase_done  phase=Done  tlog_len=14
```

### Multiple parallel agents

```sh
AGENT_COUNT=3 \
PROJECT_DIR=/path/to/project \
./target/release/agent
```

Agents 1+ start with a 5-second staggered delay. All agents share the same `PROJECT_DIR`; they coordinate through `plan.md` and `score.md`.

### Retry behaviour

Each turn attempt is logged independently. A turn is retried when:
- the `[DONE]` SSE frame was received but `message_stream_complete` is missing, **and**
- no `target_url` has been established yet (retrying after a tab is pinned would lose conversation context), **and**
- `finish_reason` is not `"length"` (truncated output cannot be safely replayed).

On exhausting `TURN_RETRY_LIMIT`, the cycle fails and sleeps before the next cycle.

---

## Single-Cycle Mode (phase-driven)

Used when no `GOAL.md` is present. Requires a running canon worker.

### Option A: Run worker directly

Terminal 1:

```sh
PORT=9091 \
AI_TLOG_DIR=tlog \
./target/release/worker
```

Terminal 2:

```sh
AI_WORKER_PORT=9091 \
AI_AGENT_DOMAIN="my task" \
AI_AGENT_METRIC="task complete" \
AI_AGENT_MAX_STEPS=20 \
./target/release/agent
```

The agent prints a final summary:

```text
agent: done  steps=<n>  success=<true|false>  stop=<reason>  phase=<phase>  tlog_len=<n>
```

`success=true` only when the worker runtime reaches `phase=Done`.

### Option B: Run through supervisor

Terminal 1:

```sh
AI_WORKER_BIN=./target/release/worker \
AI_TLOG_DIR=tlog \
AI_MCP_WORKER_URL=http://127.0.0.1:38469/mcp_worker \
SUPERVISOR_PORT=9090 \
./target/release/supervisor
```

Terminal 2:

```sh
curl http://127.0.0.1:9090/status
```

Use the worker port from the status response:

```sh
AI_WORKER_PORT=<worker-port> \
AI_AGENT_DOMAIN="my task" \
AI_AGENT_METRIC="task complete" \
AI_AGENT_MAX_STEPS=20 \
./target/release/agent
```

Hot-reload the worker after a rebuild:

```sh
curl -X POST http://127.0.0.1:9090/reload
```

### Single-cycle environment

```text
AI_WORKER_PORT          required; worker HTTP port
CANON_OPENAI_BASE_URL   optional; default http://127.0.0.1:8081/v1
CANON_OPENAI_MODEL      optional; passed through to router-server
CANON_OPENAI_TIMEOUT_MS optional; LLM request timeout
AI_AGENT_DOMAIN         optional; objective domain hint
AI_AGENT_METRIC         optional; success metric
AI_AGENT_MAX_STEPS      optional; default 20
```

### Inspect runtime state

```sh
curl http://127.0.0.1:9091/health/worker
curl http://127.0.0.1:9091/v1/state
```

The state response includes `phase`, `tlog_len`, `failure`, and `recovery_action`.
