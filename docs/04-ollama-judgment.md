# Live Ollama Judgment Path

The deterministic LLM adapter remains available for replayable tests, but the agent also includes a real local Ollama/OpenAI-compatible executable path.

```bash
export CANON_OLLAMA_BASE_URL=http://127.0.0.1:11434/v1
export CANON_OLLAMA_MODEL=qwen2.5-coder:7b
cargo run --example ollama_judgment
```

This path runs:

```text
agent runtime → observation → context → Ollama /v1/chat/completions → LlmRecord → Judgment gate → TLog receipt
```
