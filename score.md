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
I  = 6.8 / 10
E  = 6.7 / 10
C  = 6.1 / 10
A  = 8.2 / 10
R  = 6.5 / 10
P  = 5.9 / 10
S  = 5.9 / 10
D  = 7.9 / 10
T  = 8.2 / 10
Co = 7.3 / 10
Em = 7.0 / 10
B  = 6.7 / 10
L  = 6.0 / 10
Si = 5.8 / 10
F  = 7.1 / 10

G = 6.76 / 10
max(G) = good
```

Judgment: the repo is stronger as a handoff artifact because final delta manifests now preserve policy-learning and panic-surface validation evidence instead of dropping those signals after observe validation. It is still not production-ready: Rust compiler validation, wrapper graph telemetry, live Ollama execution, external API/observation tests, and semantic artifact verification remain unavailable or unproven in this container.

## Scope

```text
stage = PHASE_2_EXECUTE_TURN_002
source_changes_allowed = true
source_changes_made = true
restored_bundle = /mnt/data/ai.bundle
restored_repo_path = /mnt/data/ai-phase2-work/ai
observed_branch = main
base_commit = b7bbdbc0c30ce167e14e697ece4e928dafdcd3a0
runtime_archive = /mnt/data/ai-runtime.tar.gz
runtime_manifest_base_commit = b7bbdbc0c30ce167e14e697ece4e928dafdcd3a0
existing_goal_md = true
existing_score_md_before_phase = true
```

## Phase 2 Delta

Implemented closure:

- Updated `plan.md` from `GOAL.md` and the current score evidence.
- Extended `scripts/write_delta_manifest.py` so delta receipts and `DELTA_MANIFEST.md` preserve:
  - `policy_learning_trace_validation_result`
  - `policy_learning_trace_status`
  - `policy_learning_trace_function`
  - `policy_learning_trace_check_count`
  - `policy_learning_trace_missing_count`
  - `panic_surface_production_unwrap_count`
  - `panic_surface_production_expect_count`
  - `panic_surface_production_panic_count`
  - `panic_surface_test_total`
  - `panic_surface_example_total`
- Added regression coverage in `tests/test_write_delta_manifest.py` for manifest first-line ordering, evidence preservation, and duplicate metric prevention.
- Updated this scorecard with current evidence, current limits, and recomputed `G`.

Why this matters: `GOAL.md` requires replayable, auditable learning and policy promotion. The observe pipeline already produces the relevant evidence. The receiver manifest is the final artifact boundary, so dropping that evidence there weakens transparency and auditability.

## Existing File Contents Observed

`GOAL.md` exists. It defines Canon Agent as a deterministic, self-improving runtime with a frozen kernel, append-only TLog, policy learning, capability-layer intelligence, and an LLM promotion ladder. The highest relevant requirement for this turn is that run history must be preserved as auditable evidence and promoted into policy without compromising replay.

The prior `score.md` existed but was stale for this exact turn. It referenced an older Phase 2 base and already identified the same class of weakness: validation and runtime evidence were present but not fully closed through current artifacts.

## Runtime Archive Evidence

`/mnt/data/ai-runtime.tar.gz` was inspected. It contains:

```text
RUNTIME_MANIFEST.json
.repo-agent-runtime/*.messages.ndjson
.repo-agent-runtime/*.downloads.ndjson
.repo-agent-runtime/*.candidate-ledger.ndjson
.repo-agent-runtime/audit.ndjson
.repo-agent-runtime/current-run-summary.json
.repo-agent-runtime/delta-apply-receipts/*.json
downloads/*.md
downloads/*.ndjson
downloads/*.json
log/*.ndjson
log/ollama_judgment.results.txt
```

Current archive alignment:

```text
runtime_manifest_base_commit = b7bbdbc0c30ce167e14e697ece4e928dafdcd3a0
requested_delta_base_commit = b7bbdbc0c30ce167e14e697ece4e928dafdcd3a0
runtime_manifest_base_matches_delta_base = true
```

Important caution: runtime evidence is supporting historical state. Current-head validation still must come from commands run after this turn's changes.

## Marker Evidence

Search commands:

```bash
rg -n --hidden -g '!.git/**' -g '!target/**' -g '!patch/**' -g '!score.md' 'TODO|FIXME' .
rg -n --hidden -g '!.git/**' -g '!target/**' 'TODO|FIXME' .
```

Findings:

```text
active markers outside score.md, target, and patch archive = 0
archived patch markers = 4
score.md self-references marker evidence by design
```

Archived patch markers remain in `patch/improve_score_codebase.apply_patch`:

```text
command intake awaiting external API protocol
judgment payload awaiting versioned policy
handlers awaiting protocol schema freeze
wire encoding awaiting HTTP/gRPC transport choice
```

Critical reading: active source is clean of new deferral markers, but archived patch debt still documents unresolved external protocol/API boundary work.

## Validation Evidence

Direct validation run before commit:

```text
python3 -m unittest discover -s tests -p 'test_*.py' -v => pass, 20 tests
python3 scripts/validate_policy_learning_trace.py --root . --report target/observe/policy-learning-trace.json => pass
python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report target/observe/panic-surface.json => pass
```

Policy-learning trace result:

```text
status = pass
trace_function = learning_policy_llm_feedback_loop_drives_judgment
missing_count = 0
check_count = 4 groups / 28 token checks
```

Panic-surface result:

```text
production_total = 0
example_total = 0
test_total = 319
finding_count = 319
```

Unavailable or expected partial validation:

```text
cargo fmt --check                    => unavailable here, cargo not found
cargo test --all-targets             => unavailable here, cargo not found
cargo clippy --all-targets           => unavailable here, cargo not found
wrapper_graph_validation             => skipped unless CANON_RUSTC_WRAPPER is supplied
cargo run --example ollama_judgment  => skipped unless cargo and Ollama env are supplied
router_offline_tests                 => unavailable, router subtree missing
```

## Axis Detail

| Axis | Score | Critical basis |
|---|---:|---|
| I | 6.8 | Manifest now carries learning-policy evidence to the receiver; autonomous closed-loop reduction of LLM work is still not measured. |
| E | 6.7 | Python validation remains executable and manifest evidence is preserved without adding runtime dependency weight. |
| C | 6.1 | Added regression tests close a concrete artifact correctness gap; Rust compiler validation remains absent. |
| A | 8.2 | Change directly supports GOAL.md’s auditability and policy-learning evidence requirements. |
| R | 6.5 | Handoff robustness improves because evidence is not lost between observe report and manifest; external/runtime gaps remain. |
| P | 5.9 | No new performance load beyond small manifest serialization; no Rust benchmark or current live runtime benchmark exists. |
| S | 5.9 | Better artifact contracts scale repo-loop handoff, but orchestration/API/stream scale remain unproven. |
| D | 7.9 | Manifest first-line order and evidence fields are locked by tests. |
| T | 8.2 | Receiver can now see policy-learning and panic-surface evidence directly in `DELTA_MANIFEST.md`. |
| Co | 7.3 | Contributors get stronger final artifacts and clearer validation lineage. |
| Em | 7.0 | Operators can distinguish source-level learning validation, panic-surface safety, and unavailable compiler proof. |
| B | 6.7 | Benefit improves for deterministic repo-agent handoff; deployed user value remains unvalidated. |
| L | 6.0 | Learning evidence is preserved at the artifact boundary; actual policy promotion at runtime is still source-level here. |
| Si | 5.8 | Added fields increase manifest size, but remove a hidden evidence gap. |
| F | 7.1 | Future deltas are less likely to lose critical validation evidence during handoff. |

## Risk Register

| Risk | Severity | Evidence | Closure requirement |
|---|---:|---|---|
| Rust crate not compiler-validated here | High | `cargo` and `rustc` unavailable | Expose toolchain and rerun fmt/test/clippy. |
| Graph telemetry absent | High | no generated `state/rustc/*/graph.json` | Run explicit wrapper graph capture with `CANON_RUSTC_WRAPPER`. |
| Live Ollama proof absent at current head | High | local cargo/Ollama path unavailable | Run `cargo run --example ollama_judgment` with configured Ollama. |
| External API and observation tests absent | High | observe pipeline still flags external surface gaps | Add current-head API action and stream-ingress tests. |
| Semantic artifact verification not closed | High | observe still reports missing semantic artifact verification | Add artifact proof fixtures and enforce replay validation. |
| Monolithic Rust test surface | Medium | `src/lib.rs` remains thousands of lines with hundreds of test unwraps | Move tests into focused modules once Rust tooling is available. |
| Archived patch debt | Medium | four archived patch markers | Confirm obsolete patch status or promote unresolved protocol work to active tracked plan. |

## Next Closure Targets

1. Expose `/mnt/data/rust-sandbox/bin` or another Rust toolchain and run `cargo fmt --check`, `cargo test --all-targets`, and `cargo clippy --all-targets -- -D warnings`.
2. Generate `state/rustc/*/graph.json` with `CANON_RUSTC_WRAPPER` and record node, edge, and intent-coverage metrics.
3. Run `cargo run --example ollama_judgment` against local Ollama and record durable receipt/proof replay evidence.
4. Add executable external observation stream and API action tests.
5. Add semantic artifact verification fixtures that bind artifact digest, proof record, and replay receipt.