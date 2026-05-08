# Plan: Supervisor + Worker + MCP Integration

## Context

The `ai` crate is a Rust library plus CLI binaries. It has no HTTP server today. All state
mutation goes through `ApiTransportSession::handle_frame()` in `src/api/transport.rs`, which
writes a durable tlog to disk before touching in-memory state. The kernel is pure and has no
external dependencies.

The `chatgpt-mcp-connector` (already running) exposes:
- Public MCP supervisor: `http://0.0.0.0:4000/mcp` (OAuth-protected, proxies to worker)
- Internal MCP worker: `http://127.0.0.1:<dynamic_port>/mcp_worker` (local, no OAuth)

The goal is:
1. A stable **supervisor** binary that manages the `ai` worker process lifecycle and supports
   hot reload (for self-modification — the worker rewrites its own source, rebuilds, signals
   the supervisor to restart it).
2. A reloadable **worker** binary that owns the kernel state machine and serves it over HTTP.
3. An MCP executor in `src/capability/tooling/record/mcp.rs` that calls the local
   `mcp_worker` endpoint to execute tools (shell, apply_patch, etc.).

The supervisor/worker split mirrors the existing `chatgpt-mcp-connector` pattern exactly.
No new architectural concepts are introduced.

---

## New files

```
src/bin/supervisor.rs
src/bin/worker.rs
src/api/server.rs
src/capability/tooling/record/mcp.rs
```

## Changed files

```
Cargo.toml                              add tokio, axum, serde, serde_json, reqwest
src/capability/tooling/record/mod.rs    pub mod mcp + re-exports
src/capability/tooling/mod.rs           re-export McpCallRequest, McpCallReceipt, LiveMcpCallExecutor
src/lib.rs                              re-export new public surface
```

---

## Step 1 — Cargo.toml — completed 2026-05-08

Status: implemented in this turn. `Cargo.toml` now declares the planned HTTP/MCP
dependencies and the `supervisor` / `worker` binary targets. Minimal compiling binary
stubs exist so later turns can replace placeholder behavior with lifecycle and HTTP
server implementations while keeping `cargo check --all-targets` green.

Added dependencies. All new deps are intended for the two new binaries and future `mcp.rs`.

```toml
[dependencies]
tokio      = { version = "1", features = ["rt-multi-thread", "macros", "net", "time", "signal", "io-util"] }
axum       = { version = "0.7", features = [] }
serde      = { version = "1", features = ["derive"] }
serde_json = "1"
reqwest    = { version = "0.12", default-features = false, features = ["json", "blocking"] }

[[bin]]
name = "supervisor"
path = "src/bin/supervisor.rs"

[[bin]]
name = "worker"
path = "src/bin/worker.rs"
```

Note: `reqwest` is used by `LiveMcpCallExecutor` to call the local MCP worker. Use the
blocking client inside `execute_call()` so the executor stays sync (matching the existing
`LiveSandboxProcessExecutor` which is also sync). If the calling context is async, wrap in
`tokio::task::spawn_blocking`.

---

## Step 2 — `src/capability/tooling/record/mcp.rs` — completed 2026-05-08

Status: implemented in this turn. The MCP call request/receipt record, live allowlist-gated executor, compact NDJSON codec, append/load helpers, replay verifier, evidence submission path, public re-exports, and focused contract tests now exist. The live executor is still local-worker only and does not add kernel, policy, retrieval, or supervisor authority.

### Types

```rust
pub struct McpCallRequest {
    pub capability: CapabilityId,          // always Tooling
    pub registry_policy_hash: u64,
    pub worker_url_hash: u64,              // hash of the mcp_worker URL string
    pub tool_name_hash: u64,               // hash of tool name e.g. "shell"
    pub args_hash: u64,                    // hash of JSON args blob
    pub timeout_ms: u64,
    pub max_output_bytes: u64,
}
// contract_hash(): mix all fields, same pattern as SandboxProcessRequest

pub struct McpCallReceipt {
    pub request_hash: u64,
    pub registry_policy_hash: u64,
    pub worker_url_hash: u64,
    pub tool_name_hash: u64,
    pub args_hash: u64,
    pub timeout_ms: u64,
    pub max_output_bytes: u64,
    pub effect: Effect,                    // Effect::process(response_hash, 0, response_bytes, 0, exit_status, timed_out)
    pub response_hash: u64,                // bytes_hash of the raw JSON response body
    pub response_bytes: u64,
    pub exit_status: u64,                  // 0 = tool returned success, 1 = tool returned error
    pub timed_out: bool,
    pub receipt_hash: u64,                 // mix of all fields, same pattern as SandboxProcessReceipt
}
// is_success(): exit_status == 0 && !timed_out && is_contract_valid()
// is_contract_valid(): all hashes non-zero, effect_is_normalized()
// effect_is_normalized(): effect == Effect::process(response_hash, 0, response_bytes, 0, exit_status, timed_out)
// submission() → EvidenceSubmission::with_effect_payload(GateId::Execution, Evidence::ExecutionReceipt, is_success(), PacketEffect::None, payload_hash)
// impl EvidenceProducer for McpCallReceipt (same as SandboxProcessReceipt)
```

### Executor

```rust
pub struct LiveMcpCallExecutor {
    pub worker_url: String,                // "http://127.0.0.1:PORT/mcp_worker"
    pub allowed_tools: Vec<String>,        // allowlist — same security model as allowed_commands
    pub timeout_ms: u64,
    pub max_output_bytes: u64,
    pub registry: CapabilityRegistry,
}
```

Builder: `.with_allowed_tool("shell")`, `.with_allowed_tool("apply_patch")`,
`.with_timeout_ms(5000)`, `.with_max_output_bytes(65536)`, `.with_registry(r)`.

`execute_call(tool_name: &str, args_json: &str) -> Result<McpCallReceipt, ToolSandboxError>`:

1. Validate `tool_name` is in `allowed_tools` → `ToolSandboxError::CommandDenied`
2. Validate `args_json.len() as u64 <= max_output_bytes` → `ToolSandboxError::ArtifactTooLarge`
3. Build JSON-RPC 2.0 body:
   ```json
   {"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"<tool_name>","arguments":<args>}}
   ```
4. POST to `worker_url` with `Content-Type: application/json`, timeout = `timeout_ms`
5. On HTTP error or timeout → set `exit_status=1` / `timed_out=true`
6. Hash response body with `bytes_hash()` → `response_hash`
7. Build `Effect::process(response_hash, 0, response_bytes, 0, exit_status, timed_out)`
8. Compute `receipt_hash` (mix all fields, same pattern as `expected_process_receipt_hash`)
9. Return `McpCallReceipt`

### NDJSON codec

Same pattern as `process.rs`. Implement:
- `encode_mcp_call_receipt_ndjson(receipt: &McpCallReceipt) -> String`
- `decode_mcp_call_receipt_ndjson(line: &str) -> Result<McpCallReceipt, ToolSandboxError>`
- `append_mcp_call_receipt_ndjson(path, receipt) -> Result<(), ToolSandboxError>`
- `load_mcp_call_receipts_ndjson(path) -> Result<Vec<McpCallReceipt>, ToolSandboxError>`
- `verify_mcp_call_receipts(tlog, receipts) -> Result<usize, ToolSandboxError>`

Constants:
```rust
pub const MCP_CALL_RECEIPT_SCHEMA_VERSION: u64 = 1;
pub const MCP_CALL_RECEIPT_RECORD: u64 = 4;   // 1=tool, 2=sandbox_process, 3=process_effect, 4=mcp
```

---

## Step 3 — `src/api/server.rs`

The axum router used by the worker binary. The supervisor does not use this file.

### AppState

```rust
#[derive(Clone)]
pub struct WorkerAppState {
    inner: Arc<Mutex<WorkerSession>>,
}

pub struct WorkerSession {
    session: ApiTransportSession,
    tlog_path: PathBuf,
    next_request_id: u64,
}
```

### DTOs (serde-annotated, no kernel types exposed)

```rust
#[derive(Deserialize)]
pub struct CommandEnvelopeDto {
    pub command_id: u64,
    pub command_hash: u64,
    pub payload_tag: String,      // "SubmitEvidence" | "SubmitProcessReceipt" | "SubmitMcpReceipt" | ...
    pub payload: serde_json::Value,
}

#[derive(Serialize)]
pub struct CommandResponseDto {
    pub ok: bool,
    pub request_id: u64,
    pub event_seq: u64,
    pub event_hash: u64,
    pub phase: String,
    pub disposition: String,      // "accepted" | "replayed"
}

#[derive(Serialize)]
pub struct StateDto {
    pub phase: String,
    pub tlog_len: usize,
    pub objective_id: u64,
    pub task_id: u64,
}
```

### Routes

```
POST /v1/command   → post_command(State, Json<CommandEnvelopeDto>) → Json<CommandResponseDto>
GET  /v1/state     → get_state(State)                              → Json<StateDto>
GET  /health/worker → health()                                      → "ok"
```

`post_command`:
1. Lock `WorkerSession`
2. Allocate `next_request_id`, increment
3. Deserialize payload into `Command` based on `payload_tag`
4. Build `ApiTransportFrame::new(request_id, CommandEnvelope { command_id, command_hash, command })`
5. Call `session.handle_frame(frame)`
6. Append updated tlog to disk via `tick_durable` pattern (write before releasing lock)
7. Return `CommandResponseDto`

`get_state`: lock, read `session.state()`, return dto. No mutation.

`health`: always 200. Called by supervisor during startup wait.

### Router builder

```rust
pub fn build_router(state: WorkerAppState) -> axum::Router {
    Router::new()
        .route("/v1/command", post(post_command))
        .route("/v1/state",   get(get_state))
        .route("/health/worker", get(health))
        .with_state(state)
}
```

---

## Step 4 — `src/bin/worker.rs`

Runs when `AI_WORKER_MODE=1`. Short main:

```
1. Read PORT from env (required)
2. Read AI_TLOG_DIR from env (default: "tlog")
3. Load or initialize DurableRuntimeState from AI_TLOG_DIR
4. Build WorkerAppState wrapping ApiTransportSession::from_parts(...)
5. Build router via server::build_router(state)
6. Bind TcpListener on 127.0.0.1:PORT
7. Serve with graceful shutdown on SIGTERM/SIGINT
8. On shutdown: flush final tlog to disk, exit 0
```

The worker does NOT run its own tick loop. State advances only in response to
`POST /v1/command`. The agent loop (or curl) is the driver.

---

## Step 5 — `src/bin/supervisor.rs`

Models on `chatgpt-mcp-connector/src/worker.rs`. Stable — never reloads itself.

### WorkerProcess state

```rust
struct WorkerProcess {
    active: Option<WorkerInstance>,
    retired: Vec<RetiredWorker>,
    next_generation: u64,
    binary_path: PathBuf,          // path to the `worker` binary
}

struct WorkerInstance {
    child: tokio::process::Child,
    port: u16,
    generation: u64,
}

struct RetiredWorker {
    instance: WorkerInstance,
    retired_at: Instant,
}
```

### Supervisor routes

```
POST /reload       → reload_inner(): spawn new worker, retire old (30s drain), return generation info
GET  /health       → { "ok": true, "generation": N, "worker_port": PORT }
```

The supervisor does NOT expose `/v1/command` or `/v1/state`. Those live on the worker.
Callers that need to submit commands talk directly to the worker port (discoverable via
`GET /health` on the supervisor). This avoids double-proxying and matches the connector pattern.

### spawn_worker

```rust
async fn spawn_worker(binary_path, generation, tlog_dir, mcp_worker_url) -> Result<WorkerInstance> {
    let port = allocate_port()?;    // bind :0, read port, drop listener
    let child = Command::new(binary_path)
        .env("AI_WORKER_MODE", "1")
        .env("PORT", port.to_string())
        .env("AI_TLOG_DIR", tlog_dir)
        .env("AI_MCP_WORKER_URL", mcp_worker_url)
        .env("AI_WORKER_GENERATION", generation.to_string())
        .kill_on_drop(true)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;
    wait_for_health(port).await?;   // poll GET /health/worker, 10s deadline, 100ms interval
    Ok(WorkerInstance { child, port, generation })
}
```

### reload flow

```
POST /reload received
→ spawn new worker (new port, next_generation)
→ wait_for_health on new worker
→ update active worker pointer
→ push old worker to retired list
→ background task: after 30s, kill retired workers
→ return { ok, active_generation, active_port }
```

The worker binary path is the same binary each time. After self-modification rebuilds the
binary at `target/debug/worker` (or wherever), the next reload picks up the new binary
automatically — no path change needed.

### main

```
1. Read SUPERVISOR_PORT from env (default: 9100)
2. Read AI_TLOG_DIR from env (default: "tlog")
3. Read AI_WORKER_BIN from env (default: path to `worker` binary next to supervisor)
4. Read AI_MCP_WORKER_URL from env (default: "http://127.0.0.1:38469/mcp_worker")
5. Wrap WorkerProcess in Arc<Mutex<>>
6. spawn_worker() for generation 1
7. Start axum on 0.0.0.0:SUPERVISOR_PORT with /reload + /health routes
8. Serve until SIGTERM/SIGINT
9. On shutdown: kill active worker, exit 0
```

---

## Step 6 — Wire re-exports

`src/capability/tooling/record/mod.rs`:
```rust
pub mod mcp;
pub use self::mcp::{
    McpCallReceipt, McpCallRequest, LiveMcpCallExecutor,
    append_mcp_call_receipt_ndjson, load_mcp_call_receipts_ndjson,
    verify_mcp_call_receipts, MCP_CALL_RECEIPT_RECORD, MCP_CALL_RECEIPT_SCHEMA_VERSION,
};
```

`src/capability/tooling/mod.rs` and `src/lib.rs`: add the same re-exports to the public surface,
following the exact pattern of `SandboxProcessReceipt` and `LiveSandboxProcessExecutor`.

---

## Step 7 — Example trace

Add `examples/ollama_mcp_tool_loop_trace.rs` to demonstrate the full loop:

```
1. Read AI_MCP_WORKER_URL from env
2. Build LiveMcpCallExecutor with allowed tools: ["shell", "apply_patch"]
3. Run same judgment + tool call loop as ollama_tool_loop_trace.rs
4. For each tool call:
   a. Ask Ollama for tool intent (or use structured tool_calls if model supports it)
   b. executor.execute_call(tool_name, args_json) → McpCallReceipt
   c. append_mcp_call_receipt_ndjson to tlog dir
   d. receipts.push(receipt)
5. Command::SubmitProcessReceiptBatch → handle_envelope (McpCallReceipt implements EvidenceProducer)
6. verify_tlog + write_tlog_ndjson
```

`McpCallReceipt` submits via the same `Evidence::ExecutionReceipt` / `GateId::Execution`
path as `SandboxProcessReceipt`. The kernel sees no difference.

---

## Invariants to preserve

- The library crate (`src/lib.rs`) must compile with zero HTTP/async deps. The new tokio/axum/reqwest
  deps are used only in `src/bin/*` and `src/capability/tooling/record/mcp.rs`. If reqwest
  blocking adds unwanted transitive deps to the lib, gate it behind a `mcp` feature flag.
- All new hash functions must follow the `mix()` pattern already in the codebase. No `std::hash`,
  no external hasher.
- `McpCallReceipt::receipt_hash` must be computed identically on every re-encode/decode
  roundtrip. Write a unit test that encodes, decodes, and asserts the receipt_hash is unchanged.
- The supervisor must not crash if the worker exits unexpectedly. `try_wait()` on the child
  before forwarding to it; if dead, return 503.
- `wait_for_health` deadline: 10 seconds, 100ms poll interval — same as connector.
- Retired worker drain: 30 seconds — same as connector.

---

## What is NOT in this plan

- OAuth / `mcp_auth.rs`: not needed for local `mcp_worker` endpoint.
- `https://cheese-server.duckdns.org/ai/mcp`: the external URL is a reverse proxy to the
  local supervisor. The `ai` project talks to the local worker directly.
- Autonomous tick loop: the supervisor does not drive the kernel. Commands come in over HTTP.
- Teacher-student distillation: separate concern, not in this plan.
- `ToolKind::McpCall`: `McpCallReceipt` submits via the existing evidence path without
  needing a new ToolKind variant.
