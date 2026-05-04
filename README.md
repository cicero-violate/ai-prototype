# Canon Agent

Deterministic agent-runtime prototype with a frozen Rust kernel, typed
capabilities, replayable logs, and an embedded router-server validation target.

## Restore

```bash
git clone /path/to/ai.bundle ai
cd ai
```

Use the restored commit as delta base unless a task supplies another base:

```bash
B="$(git rev-parse HEAD)"
```

## Validate What Is Available

```bash
CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz bash scripts/observe_validation.sh
```

The observe report is written to:

```text
target/observe/validation-report.ndjson
```

This sandbox can validate Git cleanliness, router syntax checks, four offline
router behavior contracts, and runtime archive evidence. Root Rust validation
remains unavailable when `cargo`, `rustc`, or the configured wrapper path are
missing.

## Root Rust Validation

Run these when the Rust toolchain is present:

```bash
RUSTC_WORKSPACE_WRAPPER="" cargo fmt --check
RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets
RUSTC_WORKSPACE_WRAPPER="" cargo clippy --all-targets -- -D warnings
```

Run the local LLM path only with an explicit local endpoint:

```bash
CANON_OLLAMA_BASE_URL=http://127.0.0.1:11434/v1 \
CANON_OLLAMA_MODEL=qwen2.5-coder:7b \
cargo run --example ollama_judgment
```

## Delta Artifacts

After committing work:

```bash
B=<base-commit>
H="$(git rev-parse HEAD)"
rm -f /mnt/data/repo-delta-004.bundle /mnt/data/DELTA_MANIFEST.md
git bundle create /mnt/data/repo-delta-004.bundle "$B..$H"
git bundle verify /mnt/data/repo-delta-004.bundle
python3 scripts/write_delta_manifest.py \
  --base "$B" \
  --head "$H" \
  --report target/observe/validation-report.ndjson \
  --bundle /mnt/data/repo-delta-004.bundle \
  --bundle-verify pass \
  --out /mnt/data/DELTA_MANIFEST.md \
  --receipt-out target/observe/delta-validation-receipt.json
```

Receiver:

```bash
git fetch ./repo-delta-004.bundle HEAD && git merge --ff-only FETCH_HEAD
```

## Current Constraints

- Do not treat router offline tests as proof of the root Rust crate.
- Do not score graph telemetry until `state/rustc/*/graph.json` exists.
- Do not score Ollama judgment until the example runs and its receipt verifies.
- Treat downloaded manifests with base mismatches as stale advisory evidence.