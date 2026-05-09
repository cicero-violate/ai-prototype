# Agent Usage

```rust
let router = RouterClient::from_env()?;
let worker = WorkerClient::from_env()?;
let objective = AgentObjective::new("my domain", "success metric");
let summary = AgentCycle::new(router, worker, objective).run()?;
```

Env vars: `CANON_OPENAI_BASE_URL` (default `http://127.0.0.1:8081/v1`), `AI_WORKER_PORT`.
