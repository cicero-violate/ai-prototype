# Plan: Supervisor / Worker / MCP Integration

## Implementation snapshot — 2026-05-08

This repository is past the bootstrap portion of the supervisor/worker/MCP plan. The current
implementation has cargo targets, MCP receipt records, a worker API server, a reload supervisor,
and public re-exports in place. The next useful implementation turn should add committed
end-to-end evidence rather than more structural scaffolding.

Current recommended lane after this turn:

```text
continue Step 7: supervised worker command-ingress smoke, then optional example trace
```

The core gap is evidence that the already implemented surfaces work together as a single loop:

```text
supervisor -> worker health discovery -> worker command ingress -> MCP/tool receipt production -> evidence submission / durable tlog
```

---

## Completed steps

### Step 1 — Cargo targets and dependencies — completed

Implemented in commit `3117d05 Implement supervisor worker cargo targets`.

Completed surface:

```text
Cargo.toml declares supervisor and worker binary targets
Cargo.toml includes tokio, axum, serde, serde_json, reqwest, tower
src/bin/supervisor.rs exists
src/bin/worker.rs exists
cargo check --all-targets remained green after target introduction
```

### Step 2 — MCP call receipt records and executor — completed

Implemented in commit `8356abd Implement MCP call receipts`.

Completed surface:

```text
src/capability/tooling/record/mcp.rs exists
McpCallRequest exists
McpCallReceipt exists
LiveMcpCallExecutor exists
MCP receipt hash normalization exists
MCP NDJSON encode/decode/append/load helpers exist
verify_mcp_call_receipts exists
EvidenceProducer is implemented for McpCallReceipt
tests/mcp_receipt_contract.rs covers request, receipt, NDJSON, tamper rejection, allowlist, and bounds behavior
```

### Step 3 — Worker API server — completed

Implemented in commit `e77b1fc Implement worker API server`.

Completed surface:

```text
src/api/server.rs exists
WorkerAppState wraps ApiTransportSession state behind Arc<Mutex<_>>
GET /health/worker exists
GET /v1/state exists
POST /v1/command exists
SubmitEvidence and SubmitEvidenceBatch DTO paths are supported
unsupported command payload tags are rejected without mutation
tests/api_server_contract.rs covers server contract behavior
```

### Step 4 — Worker binary runtime startup — completed

Implemented in commit `cea5e2a Implement worker runtime startup`.

Completed surface:

```text
worker reads PORT
worker reads AI_TLOG_DIR
worker initializes or resumes durable runtime state
worker serves build_router(state) on 127.0.0.1:PORT
worker supports signal-driven shutdown
worker does not run an autonomous tick loop
tests/worker_binary_contract.rs covers help/startup health behavior
```

### Step 5 — Supervisor lifecycle and reload — completed

Implemented in commit `6dd425d Implement supervisor lifecycle reload`.

Completed surface:

```text
supervisor --help exits without requiring runtime environment
SUPERVISOR_PORT defaults to 9100
AI_TLOG_DIR defaults to tlog
AI_WORKER_BIN defaults to worker next to supervisor
AI_MCP_WORKER_URL defaults to http://127.0.0.1:38469/mcp_worker
supervisor starts generation 1 worker on a dynamically allocated local port
GET /health reports ok, active generation, and worker port
POST /reload starts replacement worker before retiring the old worker
retired workers are killed after a bounded drain window
shutdown kills active and retired workers
tests/supervisor_binary_contract.rs covers startup and reload behavior
```

### Step 6 — Public re-exports — completed

Completed in current source.

Completed surface:

```text
src/capability/tooling/record.rs re-exports MCP records/helpers
src/capability/tooling/mod.rs re-exports MCP records/helpers
src/lib.rs re-exports MCP records/helpers
downstream tests import MCP types from the crate root successfully
```

---

## Next implementation step

## Step 7 — Add committed end-to-end MCP/tool integration evidence

### Objective

Add one small, deterministic integration artifact that proves the current supervisor/worker/MCP
surfaces compose. Prefer a test first; add an example trace only if the test would need live
external services.

Recommended implementation order:

```text
1. Add a supervised worker command-ingress smoke test.
2. Add an MCP receipt live-executor smoke using a tiny local test HTTP server. [completed 2026-05-08]
3. Optionally add examples/ollama_mcp_tool_loop_trace.rs after the deterministic tests pass.
```

### 7A — Supervised worker command-ingress smoke

Add or extend `tests/supervisor_binary_contract.rs` with a focused test that:

```text
starts supervisor with temp AI_TLOG_DIR
polls GET /health
extracts worker_port
calls GET http://127.0.0.1:{worker_port}/health/worker
calls GET http://127.0.0.1:{worker_port}/v1/state
posts a minimal valid /v1/command envelope to the worker, if a stable DTO fixture is available
asserts the worker remains healthy and state/tlog response is coherent
terminates supervisor and verifies process cleanup
```

If a stable command fixture is not available in one turn, keep the smoke to worker discovery,
worker health, and state read. Do not fabricate command hashes or accept lossy assertions.

### 7B — LiveMcpCallExecutor local HTTP smoke — completed 2026-05-08

Added deterministic test `mcp_executor_calls_local_worker_and_records_receipt` in
`tests/mcp_receipt_contract.rs`.

Implemented evidence:

```text
starts a local HTTP server on 127.0.0.1:0
accepts JSON-RPC tools/call requests
returns a fixed JSON body for an allowed tool
constructs LiveMcpCallExecutor with that local URL and one allowed tool
executes the call
asserts receipt success, response hash normalization, args hash stability, and NDJSON roundtrip
asserts disallowed tool remains denied
```

This test does not require the external `chatgpt-mcp-connector` service and proves that
`LiveMcpCallExecutor::execute_call` performs a real local HTTP round trip, records a normalized
successful `McpCallReceipt`, validates the receipt against the original request, roundtrips
through NDJSON, and keeps disallowed tools denied.

Validation evidence:

```text
CARGO_BUILD_RUSTC_WRAPPER= cargo test --test mcp_receipt_contract --quiet
result: 6 passed
```

### 7C — Optional example trace

Add `examples/ollama_mcp_tool_loop_trace.rs` only after 7A or 7B is committed. The example may
depend on `AI_MCP_WORKER_URL`, but it must not be the only evidence for correctness because live
Ollama and connector availability are environment-dependent.

Example behavior:

```text
read AI_MCP_WORKER_URL
build LiveMcpCallExecutor with allowed tools: shell, apply_patch
execute one bounded tool call or model-selected tool call
append McpCallReceipt NDJSON
submit the receipt through the existing EvidenceProducer path
verify receipt/tlog consistency
```

---

## Guardrails for the next turn

```text
do not change kernel semantics for Step 7
do not add OAuth or external MCP auth code
do not make the supervisor proxy /v1/command or /v1/state
do not require a live external MCP endpoint in tests
do not claim a score increase without committed validation evidence
preserve unrelated worktree changes outside the Step 7 scope
```

---

## Validation commands for next implementation turn

Run the smallest relevant set first, then the broader suite:

```text
CARGO_BUILD_RUSTC_WRAPPER= cargo fmt --check
CARGO_BUILD_RUSTC_WRAPPER= cargo test --test mcp_receipt_contract --quiet
CARGO_BUILD_RUSTC_WRAPPER= cargo test --test supervisor_binary_contract --quiet
CARGO_BUILD_RUSTC_WRAPPER= cargo test --test worker_binary_contract --quiet
CARGO_BUILD_RUSTC_WRAPPER= cargo test --test api_server_contract --quiet
CARGO_BUILD_RUSTC_WRAPPER= cargo check --all-targets --quiet
CARGO_BUILD_RUSTC_WRAPPER= cargo test --test planning_contract --test score_contract --quiet
```

---

## Open risks

```text
Correctness: no committed smoke currently proves a command submitted to a supervised worker path
Transparency: no committed example currently shows MCP receipt production in a full loop trace, but deterministic live-executor receipt evidence now exists
Robustness: supervisor unexpected-worker-exit behavior should be tested before expanding authority
Simplicity: avoid turning Step 7 into a broad autonomous loop implementation
```

---

## Out of scope for the next turn

```text
teacher-student distillation
OAuth / remote MCP authentication
autonomous kernel tick loop
new ToolKind variant for MCP calls
large graph-editor or autorefactor changes
policy authority expansion
retrieval write expansion
```