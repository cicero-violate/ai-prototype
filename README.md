# Canon Agent

Deterministic agent-runtime prototype: frozen Rust kernel, typed capability
surface, replayable logs, and router-server validation evidence.

## Restore

```bash
git clone /path/to/ai.bundle ai
cd ai
B="$(git rev-parse HEAD)"
```

## Observe

```bash
CANON_DELTA_BASE="$B" \
CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz \
CANON_OBSERVE_REPORT=target/observe/validation-report.ndjson \
bash scripts/observe_validation.sh
```

The observe report is one NDJSON stream with Git hygiene, base-to-head hygiene,
router tests, Rust command status, wrapper evidence, compact runtime archive
counts, graph presence, Ollama status, and missing-signal flags.

## Rust Checks

Run when a Rust toolchain exists. Wrapper variables are cleared so root checks
never depend on local graph-capture tooling.

```bash
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo fmt --check
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo clippy --all-targets -- -D warnings
```

Capture graph telemetry only when the wrapper is explicitly available:

```bash
CANON_RUSTC_WRAPPER=/path/to/canon-rustc-v3 \
CANON_RUSTC_V2_ARTIFACT_DIR=state/rustc \
cargo test --all-targets
```

Run the live local LLM path only with an explicit local endpoint:

```bash
CANON_OLLAMA_BASE_URL=http://127.0.0.1:11434/v1 \
CANON_OLLAMA_MODEL=qwen2.5-coder:7b \
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" \
cargo run --example ollama_judgment
```

## Delta Artifacts

```bash
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

## Constraints

- Router offline tests do not prove the root Rust crate.
- Graph telemetry is absent until `state/rustc/*/graph.json` exists.
- Ollama judgment is absent until the example runs and receipt replay passes.
- Runtime archives are evidence; token and signed URL caches must stay out.