//! Compatibility shim for the process scheduler wave runner.

use crate::service::agent::AgentLoopConfig;

pub(super) fn run_wave(config: &AgentLoopConfig, tag: &str, cycle_num: u64) -> bool {
    crate::service::scheduler::run_wave(config, tag, cycle_num)
}
