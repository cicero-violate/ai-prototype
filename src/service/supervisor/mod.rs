//! Supervisor process orchestration and runtime lifecycle.

pub mod config;
pub mod process;
pub mod runtime;
pub mod state;

pub use crate::runtime::WorkspaceView as WorkspaceConfig;
pub use config::SupervisorConfig;
pub use process::{
    ActiveWorkerDto, AgentStatusDto, HealthDto, PlanStatusDto, ReloadDto, RestartDto, SpawnDto,
    SpawnRequest, StartLoopDto, StartLoopRequest, TaskClaimDto, TaskClaimRequest, TaskCompleteDto,
    TaskCompleteRequest, TaskFailDto, TaskFailRequest, TaskHeartbeatDto, TaskHeartbeatRequest,
    TaskNextDto, WorkerProcess,
};
pub use runtime::run;
pub use state::{ErrorDto, NativeMcpSession, NativeMcpState, SupervisorState};
