# Canon Agent Scorecard

## Variables

```text
I  = Intelligence
E  = Efficiency
C  = Correctness
A  = Alignment
R  = Robustness
P  = Performance
S  = Scalability
D  = Determinism
T  = Transparency
Co = Collaboration
Em = Empowerment
B  = Benefit
L  = Learning
Si = Simplicity
F  = Future-Proofing
G  = Goodness
```

## Equation

```text
G = (I*E*C*A*R*P*S*D*T*Co*Em*B*L*Si*F)^(1/15)
G = 7.07
max(G) = good
```

Goodness is the geometric mean of all 15 dimensions; one weak dimension lowers the whole system.

## Phase 1 Review Scope

```text
repository = ai
restored_bundle = /mnt/data/ai.bundle
restored_repo_path = /mnt/data/ai-phase1/ai
branch = main
head_commit = c5e91b717c253bf1383ed3344d0e4ffd2c5b1cd0
tracked_files = 83
rust_files = 58
python_test_files = 3
score_md_updated = true
source_changes_made = false
```

`GOAL.md` exists and defines Canon Agent as a deterministic, self-improving agent runtime with a frozen kernel, append-only replayable TLog, bounded recovery, capability-layer intelligence, policy learning, verified evolution, and LLM promotion from routine reasoner to novelty specialist.

Prior `score.md` existed, but it contained stale Phase 2 evidence tied to a different restored head and older validation context. This scorecard replaces it with a Phase 1 review of the restored bundle at `c5e91b717c253bf1383ed3344d0e4ffd2c5b1cd0`.

## Evidence Summary

Repository shape:

```text
tracked_files = 83
rust_files = 58
largest_rust_file = src/lib.rs, 3843 lines
large_llm_modules = src/capability/llm/openai.rs, 1970 lines; src/capability/llm/ollama.rs, 1930 lines
capability_modules = context, eval, judgment, learning, llm, memory, observation, orchestration, planning, policy, tooling, verification
```

Positive evidence:

- `src/lib.rs` and `src/main.rs` both declare `#![forbid(unsafe_code)]`.
- The source layout matches the goal structure: kernel, codec, runtime, capability, and API surfaces are separated under `src/`.
- `cargo check --offline` passed under the bootstrapped nightly toolchain.
- Library unit tests passed: `107/107`.
- Python regression tests passed: `25/25`.
- Policy-learning trace validation passed with `missing_count = 0`.
- Panic-surface validation passed for production Rust code with `production_total = 0`.
- `cargo run --example loop_trace --offline` completed a deterministic 31-event run from `Delta` to `Done` with `success=true`.
- Core surfaces expose replay, verification, durable runtime, policy promotion, distillation rows, command ledger, capability registry, and LLM receipt/proof bindings.

Negative evidence:

- `cargo fmt --check` could not run because the extracted toolchain does not include `cargo-fmt` / `rustfmt`.
- `cargo clippy --all-targets` could not run because the extracted toolchain does not include `clippy`.
- `cargo test --all-targets --offline` reached the example test binaries and then exceeded the execution limit; only the library suite is fully proven.
- No wrapper graph telemetry was observed under `state/rustc/*/graph.json`.
- Live Ollama/OpenAI network paths were not exercised; tests prove request/receipt/proof structure, not live provider behavior.
- The API surface is deterministic route/protocol code, not a proven HTTP/gRPC transport.
- Simplicity is weak: `src/lib.rs` is 3843 lines and two LLM adapter modules are about 1900 lines each.
- Test/example panic surface remains high even though production panic surface is clean: `finding_count = 328`, `test_total = 327`, `example_total = 1`.

## TODO / FIXME Marker Evidence

Search command:

```bash
rg -n --hidden -g '!.git/**' -g '!target/**' -g '!score.md' 'TODO|FIXME' .
```

Finding:

```text
active TODO/FIXME markers outside .git, target, and score.md = 0
```

Critical reading: the repo does not carry explicit TODO/FIXME deferrals in active source, tests, scripts, or docs outside this scorecard. That is positive for closure discipline, but absence of markers is not proof of completion; unresolved work is visible through missing fmt/clippy tools, incomplete all-target test proof, absent graph telemetry, unproven live LLM execution, and lack of transport-level API proof.

## Validation Evidence

Rust bootstrap:

```text
script = /mnt/data/bootstrap_rustc_session.py
archive = /mnt/data/rust-nightly-x86_64-unknown-linux-gnu.tar.gz
prefix = /mnt/data/rust-sandbox
cargo_home = /mnt/data/.cargo
rustc = rustc 1.77.0-nightly (30dfb9e04 2024-01-14)
cargo = cargo 1.77.0-nightly (84976cd69 2024-01-12)
dependency_free_probe = pass
library_fetch_probe = pass
```

Repository validation:

```text
cargo check --offline = pass
cargo test --lib --offline = pass, 107 passed
python3 -m unittest discover -s tests -p 'test_*.py' -v = pass, 25 passed
python3 scripts/validate_policy_learning_trace.py --root . --report /mnt/data/ai-phase1/policy-learning-trace.json = pass
python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report /mnt/data/ai-phase1/panic-surface.json = pass, production_total 0
cargo run --example loop_trace --offline = pass, 31 events, success true
git diff --check = pass
cargo fmt --check = unavailable, no cargo-fmt in extracted toolchain
cargo clippy --all-targets --offline -- -D warnings = unavailable, no clippy in extracted toolchain
cargo test --all-targets --offline = incomplete, timeout after library tests and several example test binaries
```

## Axis Scores

| Axis | Score | Critical basis |
|---|---:|---|
| I | 7.2 | Strong typed agent model, policy promotion, distillation, replay, and LLM receipt/proof concepts; live self-improvement is still not demonstrated end to end. |
| E | 6.8 | Dependency-free crate and fast library tests help efficiency; all-target test timeout and missing fmt/clippy reduce execution efficiency. |
| C | 7.4 | `cargo check`, 107 Rust library tests, 25 Python tests, policy trace validation, panic-surface validation, and loop trace passed. Full all-target correctness remains unproven. |
| A | 8.4 | GOAL and implementation align closely around frozen kernel, TLog, bounded recovery, policy learning, verification, and LLM-as-capability. |
| R | 7.5 | Replay, durable runtime, recovery policy, command ledger, and receipt/proof modules are strong; live provider and external transport failure modes remain under-tested. |
| P | 6.2 | Loop trace and unit tests are lightweight; no benchmark suite, no provider latency proof, and all-target test timeout lower confidence. |
| S | 6.4 | Capability taxonomy and registry are scalable in structure; large modules and no transport-level proof limit operational scale. |
| D | 8.5 | Deterministic reducer, phase ordering, hash-linked TLog, replay verification, and command idempotency are central strengths. |
| T | 8.3 | GOAL, score, tests, validation scripts, trace outputs, and explicit evidence contracts make the repo unusually inspectable. |
| Co | 7.0 | Handoff files and scripts support collaboration; stale prior score context and large monolithic files increase review burden. |
| Em | 7.2 | Operators can validate core behavior with local commands and inspect proof artifacts; missing fmt/clippy/all-target closure limits safe extension. |
| B | 6.8 | The system has clear benefit for auditable autonomous execution, but user-facing deployment value is still indirect. |
| L | 6.7 | Policy-learning trace and distillation contracts exist and validate; compounding learning is not yet measured across repeated real runs. |
| Si | 5.1 | Large `src/lib.rs`, large LLM adapters, and high test unwrap/expect counts materially reduce simplicity. |
| F | 7.4 | Proof spine, typed receipts, policy versioning, and replay semantics are future-compatible; missing external integration proof remains the main risk. |

## Risk Register

| Risk | Severity | Evidence | Closure requirement |
|---|---:|---|---|
| All-target Rust tests incomplete | High | `cargo test --all-targets --offline` timed out after library tests and example binaries | Bound or split example tests, then prove all targets complete. |
| Formatting and lint proof unavailable | Medium | extracted Rust archive lacks `cargo-fmt` and `clippy` | Provide components or separate toolchain, then run fmt and clippy. |
| Live LLM path unproven | High | Ollama/OpenAI examples not run against live local endpoints | Run live provider examples and verify receipts/proof replay. |
| Wrapper graph telemetry absent | High | no `state/rustc/*/graph.json` observed | Run explicit wrapper graph capture and record node/edge/intent metrics. |
| Transport API proof missing | High | API appears route/protocol-level, not HTTP/gRPC end-to-end | Add executable transport and command-ingress integration tests. |
| Simplicity debt | Medium | `src/lib.rs` 3843 lines; `openai.rs` 1970 lines; `ollama.rs` 1930 lines | Split tests/public exports/adapters into smaller modules. |
| Test panic surface | Medium | panic validation reports `test_total = 327`, `example_total = 1` | Replace low-value unwrap/expect calls with explicit failure messages where useful. |

## Next Closure Targets

1. Add a toolchain path that includes `rustfmt` and `clippy`, then run `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`.
2. Split or bound all-target example tests so `cargo test --all-targets --offline` completes deterministically.
3. Generate Rust wrapper graph telemetry under `state/rustc/` and report graph metrics.
4. Run live Ollama and/or OpenAI-compatible examples with local endpoints and verify proof replay.
5. Reduce `src/lib.rs` and LLM adapter size by moving embedded tests and adapter internals into narrower modules.