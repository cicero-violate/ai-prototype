# Canon Agent Score

## Implementation Step 2 Scorecard - 2026-05-08

This turn implemented Step 2 of the supervisor/worker/MCP integration plan: deterministic MCP
call request/receipt records, compact NDJSON codec, append/load helpers, replay verification,
evidence submission, public re-exports, a live allowlist-gated MCP executor, and focused receipt
contract tests.

```text
turn_type = implementation_step_2_mcp_receipts
score_change_this_turn = correctness_determinism_structure_credit
commit_scope = src/capability/tooling/record/mcp.rs, src/capability/tooling/record.rs, src/capability/tooling/mod.rs, src/lib.rs, tests/mcp_receipt_contract.rs, plan.md, score.md
recommended_next_lane = worker_http_server_transport_surface
implementation_authority_change = typed MCP receipts added; no kernel authority change
policy_authority_change = none
retrieval_write_change = none
runtime_mutation_change = none
```

## Completed This Turn

```text
new file: src/capability/tooling/record/mcp.rs
new file: tests/mcp_receipt_contract.rs
modified: src/capability/tooling/record.rs
modified: src/capability/tooling/mod.rs
modified: src/lib.rs
modified: plan.md
modified: score.md
```

Implemented surfaces:

```text
McpCallRequest
McpCallReceipt
LiveMcpCallExecutor
encode_mcp_call_receipt_ndjson
decode_mcp_call_receipt_ndjson
append_mcp_call_receipt_ndjson
load_mcp_call_receipts_ndjson
verify_mcp_call_receipts
MCP_CALL_RECEIPT_SCHEMA_VERSION
MCP_CALL_RECEIPT_RECORD
```

The MCP receipt path mirrors the existing sandbox process receipt normal form:

```text
request -> allowlist/authorization -> MCP JSON-RPC call -> Effect::process -> receipt -> evidence submission
```

The live executor is intentionally bounded:

```text
allowed_tools gate = required
timeout_ms = enforced through reqwest blocking client
max_output_bytes = enforced for args and response body
invalid JSON args = InvalidCommand
HTTP non-success = receipt with exit_status 1
network timeout = receipt with timed_out true
```

## Validation Evidence

```text
CARGO_BUILD_RUSTC_WRAPPER= cargo fmt --check                                      pass
CARGO_BUILD_RUSTC_WRAPPER= cargo test --test mcp_receipt_contract --quiet          pass
CARGO_BUILD_RUSTC_WRAPPER= cargo check --all-targets --quiet                       pass
CARGO_BUILD_RUSTC_WRAPPER= cargo test --test planning_contract --test score_contract --quiet  pass
```

Focused contract result:

```text
mcp_receipt_contract: 5 passed
planning_contract:    2 passed
score_contract:       5 passed
```

## Contract Coverage Added

```text
mcp_request_hash_is_stable_and_admissible
mcp_receipt_normalizes_process_effect
mcp_receipt_roundtrips_ndjson_and_persists
mcp_receipt_rejects_tampered_hash_and_shape
mcp_executor_enforces_allowlist_and_args_bound
```

These tests prove deterministic request hashing, normalized `Effect::process(response_hash, 0,
response_bytes, 0, exit_status, timed_out)`, stable encode/decode/persist behavior, rejection of
tampered receipts, and allowlist/size denial behavior without relying on a live MCP endpoint.

## Observed Unscored Worktree Changes

The repository still contains unrelated pre-existing/unscored worktree changes outside this
turn's implementation scope:

```text
modified: graph-editor/Cargo.toml
modified: graph-editor/src/graph.rs
modified: graph-editor/src/lib.rs
deleted:  plan-autorefactor.md
modified: tests/fixtures/validation_command_footprint_receipts.txt
modified: tests/fixtures/validation_duration_planning_receipts.txt
modified: tests/validation_harness_contract.rs
untracked: canon-rustc-v3/validation/auto_refactor_ops.py
untracked: canon-rustc-v3/validation/auto_refactor_ops_smoke.py
untracked: graph-editor/src/autorefactor.rs
untracked: graph-editor/src/bin/auto_refactor_plan.rs
untracked: src/domain/
untracked: teacher-student.md
```

These are not scored here and should remain outside the Step 2 commit.

## Current Axis Scores

A small increase is justified for Correctness, Determinism, and Structure because Step 2 adds
validated typed receipts, effect normalization, stable NDJSON roundtrips, and public boundaries.
No Robustness increase is claimed for supervisor/worker lifecycle because HTTP serving and reload
behavior are not implemented yet.

```text
I  Intelligence      = 0.98
E  Efficiency        = 0.97
C  Correctness       = 0.91
A  Alignment         = 0.97
R  Robustness        = 0.96
P  Performance       = 0.95
S  Scalability       = 0.98
D  Determinism       = 0.98
T  Transparency      = 0.98
Co Collaboration     = 0.95
Em Empowerment       = 0.95
B  Benefit           = 0.97
L  Learning          = 1.00
St Structure         = 0.99
Si Simplicity        = 0.97
F  Future-Proofing   = 0.98
```

Approximate geometric mean:

```text
G ≈ 0.969
```

## Current Judgment

```text
weakest_axis = Correctness
weakest_axis_reason = MCP receipts are validated, but worker HTTP command ingestion and supervisor lifecycle remain pending
primary_next_axis = Robustness
secondary_next_axis = Correctness
guard_axis = Determinism
current_gap = worker API server has not been implemented around ApiTransportSession
next_action = implement Step 3 src/api/server.rs with DTOs, state wrapper, health, state, and command route
score_freeze_reason = no HTTP server, worker lifecycle, reload, or live integration smoke evidence yet
```

## Conditions For Next Score Increase

The next implementation should not increase Robustness or Correctness unless it adds committed
worker API server code and focused tests proving:

```text
GET /health/worker returns stable success
GET /v1/state is read-only
POST /v1/command routes through ApiTransportSession::handle_frame
request IDs are monotonic inside WorkerSession
replayed command frames do not mutate state
invalid DTOs do not mutate state
no route bypasses the existing deterministic transport boundary
```
