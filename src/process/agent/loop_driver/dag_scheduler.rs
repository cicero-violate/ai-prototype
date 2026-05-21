//! Compatibility shim for the process scheduler wave runner.

use crate::process::agent::AgentLoopConfig;

pub(super) fn run_wave(config: &AgentLoopConfig, tag: &str, cycle_num: u64) -> bool {
    crate::process::scheduler::run_wave(config, tag, cycle_num)
}
