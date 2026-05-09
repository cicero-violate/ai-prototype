# Option A — build release
cargo build --release

# Option B — run the debug build
AI_WORKER_BIN=./target/debug/worker \
AI_TLOG_DIR=tlog \
AI_MCP_WORKER_URL=http://127.0.0.1:38469/mcp_worker \
SUPERVISOR_PORT=9090 \
./target/debug/supervisor
