cargo build
python3 canon-rustc-v3/validation/semantic_spine.py \
  --graph state/rustc/ai
cargo test
cargo run --example ollama_tool_loop_trace
cargo run --example openai_tool_loop_trace
