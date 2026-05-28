//! Dispatch adapter support.

pub mod action;
mod reasoning_trace;
pub mod task_client;
pub mod task_runner;

pub use action::dispatch_action_request;
pub use reasoning_trace::REASONING_TRACE_FILE;
