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

One-line explanation: Goodness is the geometric mean of all 15 dimensions; one weak dimension lowers the whole system.

## Score Summary

```text
I  = 6.6 / 10
E  = 6.6 / 10
C  = 5.8 / 10
A  = 8.0 / 10
R  = 6.1 / 10
P  = 5.8 / 10
S  = 5.9 / 10
D  = 7.7 / 10
T  = 7.5 / 10
Co = 7.3 / 10
Em = 6.9 / 10
B  = 6.6 / 10
L  = 5.4 / 10
Si = 5.8 / 10
F  = 7.0 / 10

G = 6.56 / 10
max(G) = good
```

Judgment: this is a disciplined deterministic-runtime prototype with strong audit goals and improving handoff safety. It is still not a reproducibly validated Rust runtime in this sandbox because `cargo`, `rustc`, graph telemetry, and current-head Ollama proof are absent. This phase improves the repository loop itself by making stale or full-history bundles fail manifest verification.

## Scope

```text
stage = PHASE_2_EXECUTE_TURN_002
source_changes_allowed = true
source_changes_made = true
rust_source_files_changed = false
scorecard_updated = true
restored_bundle = /mnt/data/ai.bundle
restored_repo_path = /mnt/data/ai-phase2-work/ai
observed_branch = main
base_commit = bbcaa3947d447396ee3599b63a9af8125b95b2a2
runtime_archive = /mnt/data/ai-runtime.tar.gz
existing_goal_md = true
existing_score_md_before_phase = true
```

## Phase 2 Turn 002 Delta

Implemented closure:

- `scripts/write_delta_manifest.py` now parses `git bundle verify` output for required refs.
- A bundle that exposes `H` but does not require the requested base commit `B` is rejected.
- Delta receipts and manifests now include `bundle_required_refs` and `bundle_requires_base_commit`.
- `tests/test_write_delta_manifest.py` now covers complete-history bundle rejection.
- The delta manifest tests now share one temporary git fixture, reducing test runtime while preserving coverage.
- `plan.md` now targets artifact handoff correctness rather than the already-closed default wrapper portability issue.

Remaining critical limits:

- `cargo` and `rustc` are unavailable in this sandbox, so Rust fmt/test/clippy remain unexecuted here.
- `state/rustc/*/graph.json` is absent, so semantic graph telemetry remains missing.
- `examples/ollama_judgment.rs` was not re-run at current HEAD because no local Ollama environment is configured.
- Runtime archive evidence is useful but historical; it cannot replace current-head validation.
- Existing TODO/FIXME markers remain only in an archived patch file, not active source.

## Existing File Contents Observed

`GOAL.md` exists and defines Canon Agent as a deterministic, self-improving agent runtime with a frozen state-machine kernel, append-only replayable TLog, capability-layer intelligence, policy learning, and an LLM promotion ladder. It explicitly requires replayable and auditable decisions, recoveries, and outcomes.

`score.md` existed before this phase and already recorded the Phase 2 portability fix: the default absolute `rustc-wrapper` had been removed, graph capture became explicit through `CANON_RUSTC_WRAPPER`, and Python validation had passed while Rust validation remained unavailable.

## Repository Evidence

```text
tracked_files = 120
rust_files_src_examples = 53
python_files = 4
shell_files = 4
third_party_rust_dependencies = 0
rust_test_attrs = 103
active_source_todo_fixme_mentions = 0
archived_patch_todo_mentions = 4
panic_surface_production_total = 0
panic_surface_test_total = 319
```

Positive evidence:

- `src/lib.rs` and `src/main.rs` use `#![forbid(unsafe_code)]`.
- `Cargo.toml` declares no third-party Rust dependencies.
- The tree matches the intended architecture: `kernel`, `codec`, `runtime`, `capability`, and `api`.
- Runtime and capability modules expose TLog replay, durable runtime resume, semantic delta logic, command ledger receipts, transition verification, policy, learning, and eval surfaces.
- Python validation covers observe-validation contracts, panic-surface validation requirements, runtime performance contract fields, and delta manifest integrity.

Critical evidence:

- Root Rust validation still cannot be reproduced here because no Rust toolchain is installed in `/mnt/data` or `PATH`.
- No generated semantic graph is present under `state/rustc`.
- Runtime archive `RUNTIME_MANIFEST.json` is anchored to base commit `bbcaa3947d447396ee3599b63a9af8125b95b2a2`, but historical download history includes stale-advisory evidence, so strict current-base artifact checks are required.
- The codebase has many panic-like calls in test-classified Rust regions. Production panic surface is clean according to the Python classifier, but this remains a classifier result rather than a Rust compiler proof.

## Validation Evidence

Commands run during this phase before commit:

```text
python3 -m unittest discover -s tests -p 'test_*.py' -v
  => pass, 13 tests

python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report target/observe/panic-surface.json
  => pass, production_total=0, test_total=319, example_total=0
```

Expected unavailable commands in this sandbox:

```text
cargo fmt --check                 => unavailable
cargo test --all-targets          => unavailable
cargo clippy --all-targets        => unavailable
cargo run --example ollama_judgment => skipped without Ollama env and cargo
```

Runtime archive evidence from `/mnt/data/ai-runtime.tar.gz`:

```text
runtime_manifest_base_commit = bbcaa3947d447396ee3599b63a9af8125b95b2a2
runtime_archive_parse_status = pass
runtime_archive_log_total > 0
runtime_archive_download_total > 0
runtime_performance_signal_present = true
runtime_stale_advisory_present = true
```

## Axis Detail

| Axis | Score | Critical basis |
|---|---:|---|
| I | 6.6 | Strong typed architecture and now stronger artifact verifier logic; closed-loop autonomous intelligence is still mostly scaffolded. |
| E | 6.6 | Delta-manifest tests are faster and the verifier catches invalid artifacts earlier; Rust validation still cannot run. |
| C | 5.8 | Python tests and bundle-base rejection improve correctness; score remains capped by unavailable Rust fmt/test/clippy. |
| A | 8.0 | Work directly supports GOAL.md auditability and replayability. |
| R | 6.1 | Stale/full-history bundle handoffs are now rejected; graph and Rust validation remain missing. |
| P | 5.8 | Test fixture sharing improves local Python test cost; runtime performance evidence remains historical. |
| S | 5.9 | Safer handoff validation scales the repo loop; no concurrency or multi-agent scaling proof was added. |
| D | 7.7 | Bundle verification now proves required base ancestry for delta artifacts. |
| T | 7.5 | Manifest and receipt expose required bundle refs and base-commit proof. |
| Co | 7.3 | Receiver handoff is less ambiguous because invalid complete-history bundles fail locally. |
| Em | 6.9 | Operators get stronger apply-time evidence and clearer failure modes. |
| B | 6.6 | Benefit increases for safe repo-loop automation, but deployed agent benefit is still unproven. |
| L | 5.4 | Learning/policy modules remain present but no new learning replay trace was executed. |
| Si | 5.8 | Test fixture reuse simplifies repeated validation; broad Rust surface remains cognitively large. |
| F | 7.0 | Delta contract hardening improves future iteration safety. |

## Risk Register

| Risk | Severity | Evidence | Closure requirement |
|---|---:|---|---|
| Root Rust validation unavailable | High | `cargo`/`rustc` not found | Provide toolchain and rerun fmt/test/clippy. |
| Missing graph telemetry | High | no `state/rustc/*/graph.json` | Regenerate graph and publish node/edge/intent metrics. |
| Current-head LLM proof absent | High | Ollama example not run here | Re-run with local Ollama and record proof replay. |
| Historical artifact staleness | Medium | runtime archive contains stale-advisory history | Keep strict base/head and bundle-required-ref checks. |
| Classifier-only panic proof | Medium | panic-surface result is Python source scan | Reconfirm with Rust validation once toolchain exists. |
| Learning proof not closed | High | no observation→eval→learning replay trace | Add and validate a current-head policy promotion trace. |

## Next Closure Targets

1. Install or expose a Rust toolchain and rerun `cargo fmt --check`, `cargo test --all-targets`, and `cargo clippy --all-targets -- -D warnings`.
2. Run graph capture explicitly with `CANON_RUSTC_WRAPPER` and regenerate `state/rustc/*/graph.json`.
3. Re-run `examples/ollama_judgment.rs` at current HEAD with local Ollama and attach receipt/proof replay evidence.
4. Add a current-head observation → judgment → eval → learning → policy promotion integration trace.
5. Keep delta manifest verification strict: every bundle must expose `H`, require `B`, and pass receiver fetch/fast-forward proof.