# canon-rustc-v3 Implementation Plan

Base commit: `5f3b8a74d170ceaeac134ad621dd4df6cfd1b914`
Stage: `EXECUTE-004-2026-05-04`
Status: implemented; source changes are limited to validation contract code and documentation

## Variables

`I`=Intelligence, `E`=Efficiency, `C`=Correctness, `A`=Alignment, `R`=Robustness, `P`=Performance, `S`=Scalability, `D`=Determinism, `T`=Transparency, `Co`=Collaboration, `Em`=Empowerment, `B`=Benefit, `L`=Learning, `Si`=Simplicity, `F`=Future-Proofing.

Current score from `score.md`:

`G=(8*6*5*8*6*6*6*5*8*7*8*7*7*5*6)^(1/15)=6.44/10`

Targeted near-term lift:

`D: 5 -> 6`, `Si: 5 -> 6`, with `C` held at `5` until native Rust receipts exist.

`max(G)=good`

## Highest-Impact Target

Create a deterministic native-validation contract that blocks false proof claims before any future delta is accepted.

Reason: `score.md` shows the main weakness is not semantic intent design; it is missing executable proof. The sandbox cannot run `cargo`, `rustc`, or `rustup`, and the runtime archive contains empty validation false passes, unsigned delta receipts, turn timeouts, and missing validation durations. The highest implementable improvement now is therefore a compact gate that requires non-empty validation commands, test counts, durations, receipt hashes, and explicit native-skip reasons before a delta can be treated as validated.

## Planned Delta

1. Add a small validation contract gate. **Implemented.**
   - New file: `validation/delta_contract_gate.py`.
   - Input: `DELTA_MANIFEST.md`, optional runtime receipt report, optional native witness report.
   - Output: hash-bound `delta_contract_report.eval.json`.
   - Reject: empty validation commands, missing test count, missing validation duration, missing bundle verify command, missing base/head commit, unsigned or hashless receipt evidence.
   - Permit `pass_with_skip` only when native Rust tools are absent and the skip is explicit.

2. Extend `validation/semantic_spine.py` minimally. **Implemented.**
   - Add `--delta-contract-report`.
   - Verify report hash.
   - Reject `pass` reports with any missing native validation signal.
   - Accept `fail` reports only as rejection evidence, not as proof of correctness.

3. Add compact fixtures. **Implemented.**
   - `validation/fixtures/delta_contract/empty_validation_false_pass.eval.json`.
   - `validation/fixtures/delta_contract/native_tools_absent_skip.eval.json`.
   - `validation/fixtures/delta_contract/valid_contract.eval.json`.

4. Reduce validation verbosity where touched. **Implemented.**
   - Keep new scripts single-purpose and short.
   - Reuse existing `canonical_hash` shape.
   - Do not expand runtime/archive parsing unless required by the contract.

5. Update documentation only where it tightens execution. **Implemented.**
   - `REPRODUCIBILITY.md`: define the delta validation contract.
   - `score.md`: update only if validation evidence supports a score change.
   - Do not claim native correctness until live Rust commands run.

## Validation Commands

Portable validation available in this sandbox:

```bash
mkdir -p /mnt/data/exec-canon-rustc-v3-turn004/out
python3 validation/semantic_spine.py
python3 validation/runtime_receipt_gate.py \
  --runtime-archive /mnt/data/canon-rustc-v3-runtime.tar.gz \
  --report /mnt/data/exec-canon-rustc-v3-turn004/out/runtime_receipt_report.eval.json || true
python3 validation/semantic_spine.py \
  --runtime-receipt-report /mnt/data/exec-canon-rustc-v3-turn004/out/runtime_receipt_report.eval.json
python3 validation/delta_contract_gate.py \
  --manifest /mnt/data/DELTA_MANIFEST.md \
  --runtime-receipt-report /mnt/data/exec-canon-rustc-v3-turn004/out/runtime_receipt_report.eval.json \
  --report /mnt/data/exec-canon-rustc-v3-turn004/out/delta_contract_report.eval.json || true
python3 validation/semantic_spine.py \
  --delta-contract-report /mnt/data/exec-canon-rustc-v3-turn004/out/delta_contract_report.eval.json
! python3 validation/semantic_spine.py \
  --delta-contract-report validation/fixtures/delta_contract/empty_validation_false_pass.eval.json
python3 validation/semantic_spine.py \
  --delta-contract-report validation/fixtures/delta_contract/native_tools_absent_skip.eval.json
python3 validation/semantic_spine.py \
  --delta-contract-report validation/fixtures/delta_contract/valid_contract.eval.json
python3 validation/semantic_scale_probe.py \
  --nodes 5000 \
  --fanout 2 \
  --risk-additions 100 \
  --threshold-ms 2000 \
  --report /mnt/data/exec-canon-rustc-v3-turn004/out/semantic_scale_report.eval.json
python3 validation/performance_gate.py \
  --scale-report /mnt/data/exec-canon-rustc-v3-turn004/out/semantic_scale_report.eval.json \
  --report /mnt/data/exec-canon-rustc-v3-turn004/out/performance_report.eval.json
python3 validation/semantic_spine.py \
  --performance-report /mnt/data/exec-canon-rustc-v3-turn004/out/performance_report.eval.json
python3 validation/reproducibility_gate.py \
  --report /mnt/data/exec-canon-rustc-v3-turn004/out/reproducibility_report.eval.json
python3 validation/semantic_spine.py \
  --reproducibility-report /mnt/data/exec-canon-rustc-v3-turn004/out/reproducibility_report.eval.json
git diff --check
```

Native validation required before raising `C` above `5`:

```bash
git submodule update --init --depth 1 vendor/rust-source
cargo generate-lockfile
cargo build --locked
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
CANON_RUSTC_V3_ARTIFACT_DIR=state/rustc-live \
RUSTC_WRAPPER="$PWD/target/debug/canon-rustc-v3" \
  cargo check --locked
python3 validation/run_semantic_witness.py \
  --require-cargo \
  --report validation/semantic_witness_report.eval.json
python3 validation/semantic_preflight.py \
  --artifact-root state/rustc-live \
  --compare-artifact-root state/rustc-live-repeat \
  --require-live-replay \
  --report validation/semantic_preflight_live.eval.json
python3 validation/performance_gate.py \
  --scale-report validation/semantic_scale_report.eval.json \
  --native-overhead-report validation/native_overhead_report.eval.json \
  --require-native-overhead \
  --report validation/performance_live.eval.json
python3 validation/reproducibility_gate.py \
  --require-lockfile \
  --require-submodule \
  --require-live-validation \
  --preflight-report validation/semantic_preflight_live.eval.json \
  --witness-report validation/semantic_witness_report.eval.json \
  --report validation/reproducibility_live.eval.json
```

## Safe Git Delta Output

Use the base supplied by the EXECUTE prompt, not the plan-stage working tree:

```bash
B=<base_commit_from_execute_prompt>
H=$(git rev-parse HEAD)
TURN=004
OUT=/mnt/data

git diff --check
git status --short
git bundle create "$OUT/repo-delta-$(printf '%03d' "$TURN").bundle" "$B..$H"
git bundle verify "$OUT/repo-delta-$(printf '%03d' "$TURN").bundle"

cat > "$OUT/DELTA_MANIFEST.md" <<EOF
# DELTA_MANIFEST

base_commit: $B
head_commit: $H
turn: $TURN

validation:
- git diff --check
- python3 validation/semantic_spine.py
- python3 validation/runtime_receipt_gate.py --runtime-archive /mnt/data/canon-rustc-v3-runtime.tar.gz --report /mnt/data/exec-canon-rustc-v3-turn004/out/runtime_receipt_report.eval.json || true
- python3 validation/semantic_spine.py --runtime-receipt-report /mnt/data/exec-canon-rustc-v3-turn004/out/runtime_receipt_report.eval.json
- python3 validation/delta_contract_gate.py --manifest /mnt/data/DELTA_MANIFEST.md --runtime-receipt-report /mnt/data/exec-canon-rustc-v3-turn004/out/runtime_receipt_report.eval.json --report /mnt/data/exec-canon-rustc-v3-turn004/out/delta_contract_report.eval.json || true
- python3 validation/semantic_spine.py --delta-contract-report /mnt/data/exec-canon-rustc-v3-turn004/out/delta_contract_report.eval.json

apply:

\`\`\`bash
git fetch ./repo-delta-$(printf '%03d' "$TURN").bundle HEAD
git merge --ff-only FETCH_HEAD
\`\`\`
EOF
```

## Non-Goals

- Do not modify Rust source for this planning stage.
- Do not claim native wrapper correctness without `cargo` and `rustc` receipts.
- Do not raise `C` above `5` from portable-only validation.
- Do not treat runtime `fail` reports as success; use them only as rejection evidence.
- Do not hide missing validation behind a generic pass label.
