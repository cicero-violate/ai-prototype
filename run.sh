# # Minimal: loop + certification against a running worker
AI_WORKER_PORT=9091 \
PROJECT_DIR=. \
CANON_OPENAI_BASE_URL=http://127.0.0.1:8081/v1 \
./target/release/agent

# With supervisor auto-reload for a fresh worker each cycle
AI_WORKER_PORT=9091 \
SUPERVISOR_PORT=9090 \
PROJECT_DIR=. \
CANON_OPENAI_BASE_URL=http://127.0.0.1:8081/v1 \
./target/release/agent
