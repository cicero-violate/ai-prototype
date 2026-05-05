# Reproducibility Contract

This repository is a rustc-wrapper witness. A portable validator may prove that
the handoff contract is checked, but only a live Rust toolchain run can prove
wrapper correctness.

## States

| Surface            | Passing state                                                                           | Current bounded state                                                |
|--------------------+-----------------------------------------------------------------------------------------+----------------------------------------------------------------------|
| Toolchain          | pinned nightly such as `nightly-YYYY-MM-DD`                                             | pinned in `rust-toolchain.toml`; static validation rejects bare nightly regressions |
| Offline toolchain archive | archive hash, Python `tarfile` component inventory, and pinned-date match are recorded before native proof is trusted | `/mnt/data/rust-nightly-x86_64-unknown-linux-gnu.tar.gz` is rejection evidence: sampled inventory lacks `rustc-dev` and does not prove `nightly-2026-04-30` |
| Cargo config       | no forced `rustc = ...` or host-specific `rustc-wrapper` in repository config            | portable config is required before native proof can be trusted       |
| Lockfile           | `Cargo.lock` exists and validation uses `--locked`                                      | absent lockfile is reproducibility risk                              |
| Rust source        | `vendor/rust-source` initialized at recorded submodule commit                           | uninitialized submodule is accepted only as missing evidence         |
| Artifacts          | generated outside git, compared by normalized graph hashes                              | no committed runtime caches or signed URL data                       |
| Witness receipts   | `receipt_hash` is canonical JSON over a typed envelope binding receipt schema, graph schema, crate, node/edge counts, and replay hashes | legacy string-concatenated receipts are rejected by static validation |
| Performance        | scale gate passes and native overhead report has baseline/wrapped timings               | synthetic scale passes; native overhead is missing evidence          |
| Runtime receipts   | delta apply receipts have non-empty validation commands/tests and signed receipts       | empty validation pass claims are rejected as failures                |
| Delta manifests    | exact first-line base/head, changed files, receiver commands, bundle verification, durations, test counts, and receipt hashes are recorded | native validation may be an explicit `native_tools_absent` skip only |
| Live wrapper proof | emitted graphs replay, witness passes, overhead measured                                | portable checks must report `pass_with_skip`                         |

## Score Rules

- Do not claim production wrapper correctness without live graph emission, replay
  comparison, and wrapper overhead evidence.
- Do not force a host-specific Rust compiler path in `.cargo/config.toml`; the
  active compiler must come from the caller's toolchain environment.
- Do not claim full performance without both portable scale evidence and native
  baseline-vs-wrapper overhead evidence.
- Do not count a runtime `validation.status=pass` receipt as proof when its
  command list is empty or its test count is zero.
- Do not derive witness `receipt_hash` from a delimited string; it must hash a
  typed canonical JSON envelope so schema and field identity are replay-bound.
- Do not publish a delta manifest unless line 1 is `base_commit: ...`, line 2 is
  `head_commit: ...`, changed files match `git diff --name-only B..H`, receiver
  apply commands are present, and validation receipts include `git bundle verify`,
  elapsed times, explicit test count, and hash-bound receipt evidence.
- Do not claim full future-proofing while missing `Cargo.lock`, uninitialized
  `vendor/rust-source`, live replay, or native overhead receipts.
- Do not regress `rust-toolchain.toml` to bare `nightly`; `semantic_spine.py`
  treats date-pinned nightly plus required components as a static invariant.
- Do not treat a standalone Rust archive as native proof unless
  `validation/toolchain_archive_gate.py` records Python `tarfile` inspection
  evidence, a valid archive hash, required components including `rustc-dev`,
  and pinned-channel match evidence. Shell `tar` inventory is not accepted.
- `pass_with_skip` is acceptable for offline handoff only when every missing signal
  is explicit and hash-bound.

## Portable Validation

```bash
python3 validation/semantic_spine.py
python3 validation/semantic_preflight.py --report validation/semantic_preflight_report.eval.json
python3 validation/semantic_spine.py --preflight-report validation/semantic_preflight_report.eval.json
python3 validation/reproducibility_gate.py --require-portable-config --report validation/reproducibility_report.eval.json
python3 validation/semantic_spine.py --reproducibility-report validation/reproducibility_report.eval.json
python3 validation/semantic_scale_probe.py --report validation/semantic_scale_report.eval.json
python3 validation/performance_gate.py --scale-report validation/semantic_scale_report.eval.json --report validation/performance_report.eval.json
python3 validation/semantic_spine.py --performance-report validation/performance_report.eval.json
python3 validation/runtime_receipt_gate.py --runtime-archive /mnt/data/canon-rustc-v3-runtime.tar.gz --report validation/runtime_receipt_report.eval.json || true
python3 validation/semantic_spine.py --runtime-receipt-report validation/runtime_receipt_report.eval.json
python3 validation/delta_contract_gate.py --manifest /mnt/data/DELTA_MANIFEST.md --runtime-receipt-report validation/runtime_receipt_report.eval.json --report validation/delta_contract_report.eval.json || true
python3 validation/semantic_spine.py --delta-contract-report validation/delta_contract_report.eval.json
! python3 validation/semantic_spine.py --reproducibility-report validation/fixtures/reproducibility/false_future_proof_pass.eval.json
! python3 validation/semantic_spine.py --performance-report validation/fixtures/performance/false_performance_pass.eval.json
! python3 validation/semantic_spine.py --runtime-receipt-report validation/fixtures/runtime/empty_validation_false_pass.eval.json
! python3 validation/semantic_spine.py --delta-contract-report validation/fixtures/delta_contract/empty_validation_false_pass.eval.json
python3 validation/semantic_spine.py --runtime-receipt-report validation/fixtures/runtime/timeout_fail.eval.json
python3 validation/semantic_spine.py --runtime-receipt-report validation/fixtures/runtime/safe_skip.eval.json
python3 validation/semantic_spine.py --delta-contract-report validation/fixtures/delta_contract/native_tools_absent_skip.eval.json
python3 validation/semantic_spine.py --delta-contract-report validation/fixtures/delta_contract/valid_contract.eval.json
python3 validation/toolchain_archive_gate.py --archive /mnt/data/rust-nightly-x86_64-unknown-linux-gnu.tar.gz --require-channel-match --report validation/toolchain_archive_report.eval.json || true
python3 validation/semantic_spine.py --toolchain-archive-report validation/toolchain_archive_report.eval.json
! python3 validation/semantic_spine.py --toolchain-archive-report validation/fixtures/toolchain_archive/false_archive_pass.eval.json
python3 validation/semantic_spine.py --toolchain-archive-report validation/fixtures/toolchain_archive/mismatch_fail.eval.json
python3 validation/semantic_spine.py --toolchain-archive-report validation/fixtures/toolchain_archive/absent_skip.eval.json
```

## Full Validation

Run only in an environment with network/submodule access and the required Rust
toolchain:

```bash
git submodule update --init --depth 1 vendor/rust-source
cargo generate-lockfile
cargo build --locked
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
python3 validation/run_semantic_witness.py --require-cargo --report validation/semantic_witness_report.eval.json
python3 validation/semantic_preflight.py \
  --artifact-root state/rustc-test-1 \
  --compare-artifact-root state/rustc-test-2 \
  --require-live-replay \
  --report validation/semantic_preflight_live.eval.json
python3 validation/performance_gate.py \
  --scale-report validation/semantic_scale_report.eval.json \
  --native-overhead-report validation/native_overhead_report.eval.json \
  --require-native-overhead \
  --report validation/performance_live.eval.json
python3 validation/semantic_spine.py --performance-report validation/performance_live.eval.json
python3 validation/reproducibility_gate.py \
  --require-lockfile \
  --require-submodule \
  --require-live-validation \
  --preflight-report validation/semantic_preflight_live.eval.json \
  --witness-report validation/semantic_witness_report.eval.json \
  --report validation/reproducibility_live.eval.json
```
