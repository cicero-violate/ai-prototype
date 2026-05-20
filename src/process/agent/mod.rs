//! Canon agent — autonomous observe/decide/act/verify driver.
//!
//! Layering boundary:
//!   agent     = objective management, loop policy, step selection, driver coordination
//!   capability = evidence production, receipts, LLM/tool effects
//!   api/worker = command ingress, durable state mutation
//!   runtime   = deterministic tick/reduce/replay/recovery
//!   kernel    = frozen state machine types
//!
//! The agent submits work through CommandEnvelope / ApiTransportFrame.
//! It does not mutate State, GateSet, Packet, or TLog directly.

pub mod config;
pub mod cycle;
pub mod loop_driver;
pub mod objective;
pub mod prompt;
pub mod router;
pub mod sse;
pub mod step;
pub mod worker_client;

pub use config::{AgentLoopConfig, DEFAULT_MINI_AGENT_COUNT, MAX_MINI_AGENT_COUNT};
pub use cycle::{AgentCycle, CycleError, StopReason};
pub use loop_driver::LoopDriver;
pub use objective::AgentObjective;
pub use router::{RouterClient, RouterStreamingResult, RouterTurnResult};
pub use sse::{ChunkLogger, SseResult};
pub use step::{AgentActionKind, AgentDecision, AgentRunSummary, AgentStep};
pub use worker_client::{WorkerClient, WorkerClientError, WorkerResponse};
