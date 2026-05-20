//! Supervisor process orchestration and runtime lifecycle.

pub mod config;
pub mod process;
pub mod runtime;
pub mod state;
pub mod workspace;

pub use config::SupervisorConfig;
pub use process::{
    ActiveWorkerDto, HealthDto, ReloadDto, RestartDto, SpawnDto, SpawnRequest, StartLoopDto,
    StartLoopRequest, TaskNextDto, WorkerProcess,
};
pub use runtime::run;
pub use state::{ErrorDto, NativeMcpSession, NativeMcpState, SupervisorState};
pub use workspace::WorkspaceConfig;
