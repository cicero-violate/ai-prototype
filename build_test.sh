#!/usr/bin/env sh
set -eu

# Some sandbox runners expose /tmp with a quota even when df(1) reports free
# space. Keep Rust test temp files inside the repo's ignored target tree so
# validation does not depend on host tmpfs quota behavior.
export TMPDIR="${TMPDIR:-$PWD/target/test-tmp}"
mkdir -p "$TMPDIR"

export RUSTC_WRAPPER="${RUSTC_WRAPPER:-}"
export RUSTC_WORKSPACE_WRAPPER="${RUSTC_WORKSPACE_WRAPPER:-}"

cargo build
python3 ../canon-rustc-v3/validation/semantic_spine.py \
  --graph state/rustc/ai
cargo test
cargo run --example ollama_tool_loop_trace
cargo run --example openai_tool_loop_trace
