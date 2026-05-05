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
max(G) = good
```

Goodness is the geometric mean of all 15 dimensions; one weak dimension lowers the whole system.

## Baseline Before Source Changes

```text
repository = ai
base_commit = b903059731873bcf0be6a7d324294f786917b9be
restored_head_before_changes = b903059731873bcf0be6a7d324294f786917b9be
tracked_files = 128
rust_files = 72
runtime_archive = /mnt/data/ai-runtime.tar.gz
runtime_ndjson_files = 15
runtime_audit_events = 48
runtime_largest_message_ledger_lines = 318
```

`GOAL.md` was read from the restored repository. It requires a deterministic, self-improving agent runtime with a frozen kernel, append-only replayable TLog, bounded recovery, evaluator-scored evolution, proof-backed distillation rows, and policy learning that reduces repeated LLM work over time.

The previous scorecard was stale for this restored bundle: it referenced a different restored path/head and older validation context. The current baseline re-evaluates the repository at `b903059731873bcf0be6a7d324294f786917b9be`.

Positive evidence:

- The repository has kernel, runtime, codec, API, capability, verification, policy, learning, LLM, tooling, and nested `canon-rustc-v3/` semantic-wrapper surfaces.
- `src/lib.rs` and `src/main.rs` forbid unsafe code.
- API transport contract tests and Python validation contracts are present.
- Runtime archive inspection found message ledgers, download ledgers, audit events, delta-apply receipts, and loop-stop receipts.

Negative evidence:

- Bootstrap robustness is weak: `bootstrap_rustc_session.py` considers a prefix valid when `rustc` and `cargo` merely exist and are executable; it does not prove they run.
- Extraction writes directly into the final prefix, so timeout/interruption can leave a partial persistent Rust toolchain that later validation reuses incorrectly.
- The previous plan references older phase boundaries and stale bundle filenames.
- Large files remain: `src/lib.rs` is about 3928 lines; `openai.rs` and `ollama.rs` are about 1900 lines each.
- Wrapper graph telemetry under `state/rustc/*/graph.json` is absent in the restored root.

TODO/FIXME search:

```text
command = rg -n --hidden -g '!.git/**' -g '!target/**' -g '!score.md' 'TODO|FIXME' .
active_markers = 1
finding = canon-rustc-v3/plan.md:39 references TODO/FIXME evidence in prior score work
```

Baseline scores:

| Axis | Score | Basis |
|------|------:|-------|
| I | 7.3 | Strong typed runtime and proof concepts; current learning impact still mostly test/proof-level. |
| E | 5.8 | Validation scripts exist, but stale score/plan and non-atomic bootstrap waste operator cycles. |
| C | 6.0 | Test surfaces exist; setup can silently accept a broken toolchain. |
| A | 8.5 | Implementation structure strongly matches `GOAL.md`. |
| R | 5.8 | Timeout/interruption can corrupt bootstrap state. |
| P | 5.2 | Root crate is dependency-light; setup performance and large modules remain weak. |
| S | 6.4 | Capability taxonomy scales conceptually; operational proof is still bounded. |
| D | 7.4 | Runtime determinism is strong; environment reconstruction is less deterministic. |
| T | 7.2 | Runtime archive and validation scripts help; stale phase evidence hurts trace clarity. |
| Co | 6.5 | Handoff tooling exists; stale plan/score burden review. |
| Em | 6.1 | Operators can run scripts, but partial-prefix reuse is a trap. |
| B | 6.8 | Clear benefit for auditable autonomous execution; deployment proof remains partial. |
| L | 6.6 | Policy/distillation contracts exist; compounding learning is not yet measured on real repeated runs. |
| Si | 5.0 | Large modules and setup state complexity reduce simplicity. |
| F | 6.9 | Proof spine is future-compatible; bootstrap durability needs repair. |

```text
G = 6.44
max(G) = good
```

Weakest dimensions selected for this turn: robustness, determinism, efficiency, transparency, and operator empowerment. The one-turn target is to make Rust bootstrap reuse and extraction restart-safe, then prove that behavior with Python contract tests and rerun core validation.

## Final Score After Implementation

Implemented changes:

- `bootstrap_rustc_session.py` now proves an existing prefix by running both `rustc --version` and `cargo --version`, rather than trusting executable bits.
- Component extraction now writes into `tmp/prefix-stage` and moves the staged prefix into place only after all wanted components are seen and extracted.
- Invalid prefixes are removed before rebuild attempts, preventing known-bad state from being reused.
- Added `tests/test_bootstrap_rustc_session_contract.py` with three regression tests for broken executable rejection, failed extraction prefix preservation, and successful atomic replacement.
- Replaced stale phase evidence in `plan.md` and this scorecard with the committed one-shot boundary.

Final validation:

```text
timeout 300s python3 /mnt/data/bootstrap_rustc_session.py --skip-library-probe --no-install-launchers = pass
rustc = rustc 1.75.0 (82e1608df 2023-12-21)
cargo = cargo 1.75.0 (1d8b05cdd 2023-11-20)
CARGO_INCREMENTAL=0 timeout 300s cargo check --offline = pass
CARGO_INCREMENTAL=0 timeout 300s cargo test --all-targets --offline = pass, 110 lib tests + 11 integration tests + example/main test binaries
timeout 300s python3 -m unittest discover -s tests -p 'test_*.py' -v = pass, 30 tests
timeout 300s python3 scripts/validate_policy_learning_trace.py --root . --report /tmp/ai-one-shot-policy-learning-trace.json = pass, missing_count=0
timeout 300s python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report /tmp/ai-one-shot-panic-surface.json = pass, production_total=0, test_total=335, example_total=1, finding_count=336
CARGO_INCREMENTAL=0 timeout 300s cargo run --example loop_trace --offline = pass, 31 events, success=true
timeout 300s cargo fmt --check = unavailable, cargo-fmt not installed
timeout 300s cargo clippy --all-targets --offline -- -D warnings = unavailable, clippy not installed
git diff --check = pass
```

Final scores:

| Axis | Score | Basis |
|------|------:|-------|
| I | 7.4 | Same strong typed agent model plus improved setup self-checking. |
| E | 6.6 | Bootstrap no longer silently reuses broken prefixes; validation completed. |
| C | 7.0 | Full root all-target Rust tests, Python tests, policy trace, panic-surface, and loop trace passed. |
| A | 8.5 | Change preserves the GOAL: deterministic, auditable execution with verifiable setup. |
| R | 7.2 | Timeout/interruption state is now guarded by runtime executable checks and staged extraction. |
| P | 6.1 | Root validation completes inside 300s; fmt/clippy still unavailable. |
| S | 6.5 | Operational setup is more repeatable; graph telemetry is still absent. |
| D | 8.0 | Environment reconstruction is more deterministic because bad prefixes are rejected. |
| T | 7.9 | Score and plan now reflect actual restored head, runtime archive, changes, and validation. |
| Co | 7.0 | New tests and fresh plan reduce handoff ambiguity. |
| Em | 7.1 | Operators get safer bootstrap reuse and explicit failure behavior. |
| B | 6.9 | Benefit improves through more reliable reproducible validation. |
| L | 6.7 | Learning contracts remain validated; no new repeated-run learning metric added. |
| Si | 5.3 | Bootstrap state handling is simpler, but large modules remain. |
| F | 7.5 | Atomic setup and executable validation improve future one-shot repo turns. |

```text
G = 7.00
max(G) = good
```

Remaining risks:

- `cargo fmt` and `cargo clippy` remain unavailable in the supplied toolchain.
- Wrapper graph telemetry is still not generated under `state/rustc/*/graph.json`.
- Live Ollama/OpenAI endpoint paths were not exercised.
- Large Rust modules remain a simplicity/reviewability risk.