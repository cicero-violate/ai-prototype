<!-- 1. Loop mode only (no tlog) -->

<!-- Need: router-server running, a project with GOAL.md. -->

echo "Implement X, add tests, make CI pass." > /my/project/GOAL.md

# 2. Run
PROJECT_DIR=/my/project \
CANON_OPENAI_BASE_URL=http://127.0.0.1:8081/v1 \
./target/release/agent

<!-- Each cycle: planning turn (writes plan.md, score.md) → 5 execute turns (reads plan.md, implements, commits) → sleeps 5 s → repeat. -->

<!-- --- -->
<!-- 2. Loop + certification (tlog + receipts) -->

<!-- Need: router-server + a running worker. -->

# Terminal 1 — start worker
PORT=9091 AI_TLOG_DIR=tlog ./target/release/worker

# Terminal 2 — run agent
AI_WORKER_PORT=9091 \
PROJECT_DIR=/my/project \
CANON_OPENAI_BASE_URL=http://127.0.0.1:8081/v1 \
./target/release/agent

<!-- After each loop cycle the agent certifies the work: drives the worker through all evidence gates (Invariant → Analysis → Judgment → Plan → Execute → Verify → Eval → Done) and stamps receipts to the tlog. -->

<!-- --- -->
<!-- 3. Loop + certification + supervisor (auto-reload) -->

<!-- Need: router-server + supervisor (which manages the worker). The supervisor reloads the worker before each certification so every cycle gets a fresh state. -->

# Terminal 1 — start supervisor
AI_WORKER_BIN=./target/release/worker \
AI_TLOG_DIR=tlog \
SUPERVISOR_PORT=9090 \
./target/release/supervisor

# Terminal 2 — run agent
AI_WORKER_PORT=$(curl -s http://127.0.0.1:9090/status | grep -o '"port":[0-9]*' | grep -o '[0-9]*') \
SUPERVISOR_PORT=9090 \
PROJECT_DIR=/workspace/ai_sandbox/canon-mini-agent/prototype/ai \
CANON_OPENAI_BASE_URL=http://127.0.0.1:8081/v1 \
./target/release/agent


---
Key knobs

EXECUTE_TURNS=3        # fewer turns per cycle (default 5)
AGENT_COUNT=2          # run 2 parallel agents
LOOP_SLEEP_MS=10000    # 10 s between cycles
AI_CERT_MAX_STEPS=40   # more headroom for certification phases
TURN_RETRY_LIMIT=3     # retry incomplete turns up to 3 times

---
Key knobs

EXECUTE_TURNS=3        # fewer turns per cycle (default 5)
AGENT_COUNT=2          # run 2 parallel agents
LOOP_SLEEP_MS=10000    # 10 s between cycles
AI_CERT_MAX_STEPS=40   # more headroom for certification phases
TURN_RETRY_LIMIT=3     # retry incomplete turns up to 3 times
