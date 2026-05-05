cargo build
python3 /mnt/data/canon-mini-agent-extracted/canon-mini-agent/prototype/canon-rustc-v3/validation/semantic_spine.py \
  --graph /mnt/data/canon-mini-agent-extracted/canon-mini-agent/prototype/ai/state/rustc/ai
cargo test
cargo run --example ollama_tool_loop_trace
cargo run --example openai_tool_loop_trace
