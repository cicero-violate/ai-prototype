# Canon Agent Score

## Planning Scorecard - 2026-05-08

This planning turn reviewed the current `ai` crate and updated the active implementation
plan toward supervisor/worker/MCP integration. No runtime implementation score increase is
claimed. Existing uncommitted implementation changes outside `plan.md` and `score.md` remain
unscored and intentionally untouched.

```text
turn_type = planning_supervisor_worker_mcp
score_change_this_turn = none
commit_scope = plan.md, score.md
recommended_next_lane = supervisor_worker_mcp_integration
implementation_authority_change = none
policy_authority_change = none
retrieval_write_change = none
runtime_mutation_change = none
```

## Current Evidence Posture

The project already contains a deterministic API transport boundary:

```text
src/api/transport.rs      ApiTransportSession::handle_frame
src/api/routes.rs         command envelope handling and evidence submission
src/capability/tooling    sandbox process receipts and deterministic effect records
```

The selected next implementation should be additive:

1. add a reloadable worker HTTP surface around the existing transport session;
2. add a stable supervisor that manages worker lifecycle and reloads;
3. add an MCP tool-call receipt/executor modeled on existing sandbox process receipts;
4. preserve kernel authority, hash-chain determinism, and typed evidence boundaries.

No score movement is recorded because this turn updated planning/scoring documents only.

## Observed Unscored Worktree Changes

The worktree contains unrelated pre-existing implementation changes that were not modified by
this turn and should not be scored here:

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
untracked: teacher-student.md
```

These changes may become score-relevant only after focused validation evidence and a separate
implementation commit. They are out of scope for the supervisor/worker/MCP planning commit.

## Current Axis Scores

Scores remain unchanged for this planning checkpoint.

```text
I  Intelligence      = 0.98
E  Efficiency        = 0.97
C  Correctness       = 0.90
A  Alignment         = 0.97
R  Robustness        = 0.96
P  Performance       = 0.95
S  Scalability       = 0.98
D  Determinism       = 0.97
T  Transparency      = 0.98
Co Collaboration     = 0.95
Em Empowerment       = 0.95
B  Benefit           = 0.97
L  Learning          = 1.00
St Structure         = 0.97
Si Simplicity        = 0.97
F  Future-Proofing   = 0.98
```

Approximate geometric mean:

```text
G ≈ 0.966
```

## Current Judgment

```text
weakest_axis = Correctness
weakest_axis_reason = no supervisor, worker, or MCP implementation has been validated in this turn
primary_next_axis = Structure
secondary_next_axis = Robustness
guard_axis = Correctness
current_gap = live HTTP lifecycle and MCP receipt surfaces are planned but not implemented
next_action = implement Step 1 dependency/bin declarations and Step 2 MCP receipt types/codecs/tests
score_freeze_reason = planning-only checkpoint; no new runtime validation evidence added
```

## Conditions For Future Score Increase

Score increases are allowed only after committed implementation evidence and clean validation
output.

Suggested movement if supervisor/worker/MCP integration is implemented and validated:

```text
Structure: +0.01 if supervisor, worker, API server, and MCP receipt code are separated cleanly
Robustness: +0.01 if worker reload, health checks, dead-worker handling, and timeout paths are tested
Correctness: +0.01 only if focused contract tests prove receipt normalization and transport replay
Determinism: +0.01 only if MCP receipt encode/decode and hash roundtrips are byte-stable
Scalability: +0.01 only if lifecycle management avoids coupling the kernel to HTTP or async state
```

Do not raise any score for:

```text
uncommitted implementation
unexecuted tests
generated plans without validation
live LLM output
HTTP endpoints that bypass ApiTransportSession
MCP calls that do not produce typed receipts
policy promotion without external validation
retrieval writes without explicit storage authority
```

## Validation Required For Next Scored Turn

Minimum baseline:

```text
CARGO_BUILD_RUSTC_WRAPPER= cargo fmt --check
CARGO_BUILD_RUSTC_WRAPPER= cargo check --quiet
CARGO_BUILD_RUSTC_WRAPPER= cargo test --test planning_contract --test score_contract --quiet
```

Supervisor/worker/MCP-specific validation:

```text
CARGO_BUILD_RUSTC_WRAPPER= cargo test --test api_transport_contract --quiet
CARGO_BUILD_RUSTC_WRAPPER= cargo test --test mcp_receipt_contract --quiet
CARGO_BUILD_RUSTC_WRAPPER= cargo run --bin worker -- --help or equivalent startup smoke
CARGO_BUILD_RUSTC_WRAPPER= cargo run --bin supervisor -- --help or equivalent startup smoke
manual or scripted reload smoke: supervisor /health -> /reload -> new worker /health/worker
MCP receipt roundtrip: encode -> decode -> same receipt_hash and normalized Effect::process
```

## Non-Scored Planning Result

This checkpoint improves handoff clarity for the current implementation objective. It does not
change the numeric score because it adds no runtime implementation or validation evidence.
