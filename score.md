# Canon Agent Score

## Implementation Step 7B Scorecard — 2026-05-08

This turn implemented deterministic Step 7B evidence: a live `LiveMcpCallExecutor` smoke test
against a local one-shot HTTP server. The test exercises a real JSON-RPC `tools/call` POST,
records a successful normalized `McpCallReceipt`, validates replay against the original request,
roundtrips through NDJSON, verifies receipt shape, and confirms disallowed tools remain denied.

```text
turn_type = implementation_step_7b_live_mcp_executor_smoke
score_change_this_turn = correctness_transparency_credit
commit_scope = tests/mcp_receipt_contract.rs, plan.md, score.md
recommended_next_lane = step_7a_supervised_worker_command_ingress_smoke
implementation_authority_change = none; test evidence only
policy_authority_change = none
retrieval_write_change = none
runtime_mutation_change = none
```

---

## Current implementation state

Completed implementation commits currently visible:

```text
3117d05 Implement supervisor worker cargo targets
8356abd Implement MCP call receipts
e77b1fc Implement worker API server
cea5e2a Implement worker runtime startup
6dd425d Implement supervisor lifecycle reload
```

Completed capability surfaces:

```text
Cargo supervisor/worker targets exist
McpCallRequest, McpCallReceipt, LiveMcpCallExecutor exist
MCP receipt NDJSON helpers and verifier exist
McpCallReceipt implements EvidenceProducer
Worker API server exposes /health/worker, /v1/state, /v1/command
Worker binary starts a local HTTP server from PORT and AI_TLOG_DIR
Supervisor starts worker generation 1 and supports /health and /reload
Supervisor reload starts replacement worker before retiring old worker
MCP types/helpers are re-exported from record, tooling, and crate root surfaces
LiveMcpCallExecutor has deterministic live HTTP smoke coverage without external MCP service dependency
```

---

## Evidence from this turn

New deterministic contract added:

```text
mcp_executor_calls_local_worker_and_records_receipt
```

The new test proves:

```text
local HTTP server receives POST /mcp_worker
request body contains JSON-RPC tools/call
request body contains allowed tool shell and bounded args
executor returns successful McpCallReceipt
receipt effect is normalized
receipt validates against the original McpCallRequest
receipt replays true through LiveMcpCallExecutor::replay_receipt
receipt roundtrips through MCP NDJSON codec
verify_mcp_call_receipts accepts the decoded receipt
disallowed apply_patch call remains CommandDenied
```

Validation run in this turn:

```text
CARGO_BUILD_RUSTC_WRAPPER= cargo fmt --check                                      pass
CARGO_BUILD_RUSTC_WRAPPER= cargo test --test mcp_receipt_contract --quiet          pass, 6 tests
CARGO_BUILD_RUSTC_WRAPPER= cargo test --test supervisor_binary_contract --quiet    pass, 2 tests
CARGO_BUILD_RUSTC_WRAPPER= cargo test --test worker_binary_contract --quiet        pass, 2 tests
CARGO_BUILD_RUSTC_WRAPPER= cargo test --test api_server_contract --quiet           pass, 4 tests
CARGO_BUILD_RUSTC_WRAPPER= cargo check --all-targets --quiet                       pass
CARGO_BUILD_RUSTC_WRAPPER= cargo test --test planning_contract --test score_contract --quiet  pass, 7 tests
```

Focused contract coverage already present:

```text
mcp_receipt_contract: request hash, receipt normalization, NDJSON roundtrip, tamper rejection, executor allowlist/bounds, live local HTTP executor smoke
api_server_contract: health, state, evidence command DTO handling, unsupported command rejection
worker_binary_contract: worker help/startup health behavior
supervisor_binary_contract: supervisor help, worker startup, reload generation behavior
planning_contract: planning record invariants
score_contract: score vector, timing, recovery, validation receipt invariants
```

---

## Current axis scores

A small score increase is justified for Correctness and Transparency because one missing Step 7
condition is now satisfied by deterministic committed evidence. Robustness is held because
supervised-worker command ingress and unexpected-worker-exit coverage remain open.

```text
I  Intelligence      = 0.98
E  Efficiency        = 0.97
C  Correctness       = 0.95
A  Alignment         = 0.97
R  Robustness        = 0.99
P  Performance       = 0.95
S  Scalability       = 1.00
D  Determinism       = 0.98
T  Transparency      = 0.99
Co Collaboration     = 0.95
Em Empowerment       = 0.95
B  Benefit           = 0.97
L  Learning          = 1.00
St Structure         = 1.00
Si Simplicity        = 0.97
F  Future-Proofing   = 0.98
```

Approximate geometric mean:

```text
G ≈ 0.977
```

---

## Current judgment

```text
weakest_axis = Correctness
weakest_axis_reason = live MCP executor evidence now exists, but no committed deterministic smoke currently proves command ingress through a supervised worker
primary_next_axis = Correctness
secondary_next_axis = Robustness
guard_axis = Robustness
current_gap = missing committed Step 7A evidence connecting supervisor health discovery to worker state/command ingress
next_action = add supervised-worker command-ingress smoke without fabricating command hashes
score_freeze_reason = do not increase further until supervised worker ingress or full example trace evidence is committed
```

---

## Conditions for next score increase

The next implementation turn may increase Correctness and/or Robustness only if it commits
validation evidence proving at least one remaining gap:

```text
a supervised-worker smoke discovers worker_port from supervisor /health and exercises worker /health/worker plus /v1/state or /v1/command
an example trace records an McpCallReceipt and submits it through the existing EvidenceProducer path
the resulting receipt/tlog is verified by committed tests or reproducible commands
```

The next turn should not increase score for documentation-only changes, broad scaffolding, or
example code that requires unavailable external services and lacks deterministic test coverage.

---

## Unscored worktree changes to preserve

The current `ai` worktree contains unrelated pre-existing changes outside this planning turn.
They are not scored here and should not be included in the planning/scoring commit:

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
untracked: src/agent/
untracked: src/domain/
untracked: teacher-student.md
```

Only `plan.md` and `score.md` should be staged for this turn.