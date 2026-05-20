//! DAG wave scheduling records for kernel TLog.
//!
//! `WaveRecord`      — submitted once before a scheduling wave is spawned.
//! `ChildCompleteRecord` — submitted once per child after it joins.
//!
//! Both are observational (state_after == state_before in the kernel).
//! Together they give the TLog enough information to reconstruct which nodes
//! ran in which wave and whether they succeeded, enabling crash-resume without
//! polling plan.json.

use crate::kernel::mix;

const WAVE_RECORD_SCHEMA_VERSION: u64 = 1;
const CHILD_COMPLETE_SCHEMA_VERSION: u64 = 1;

/// Submitted by the parent scheduler before spawning a wave.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WaveRecord {
    /// Unique ID for this wave: mix of parent_hash and cycle.
    pub wave_id: u64,
    /// FNV-1a hash of the parent agent tag string.
    pub parent_hash: u64,
    /// Cycle number in which this wave is dispatched.
    pub cycle: u64,
    /// Number of child tasks in this wave (used for fan-in accounting).
    pub node_count: u16,
    /// FNV-1a hash of the sorted node IDs, binding the wave to its plan nodes.
    pub node_ids_hash: u64,
    /// Self-hash binding all fields.
    pub contract_hash: u64,
}

impl WaveRecord {
    pub fn new(
        wave_id: u64,
        parent_hash: u64,
        cycle: u64,
        node_count: u16,
        node_ids_hash: u64,
    ) -> Self {
        let contract_hash =
            expected_wave_contract_hash(wave_id, parent_hash, cycle, node_count, node_ids_hash);
        Self {
            wave_id,
            parent_hash,
            cycle,
            node_count,
            node_ids_hash,
            contract_hash,
        }
    }

    pub fn is_contract_valid(self) -> bool {
        self.wave_id != 0
            && self.parent_hash != 0
            && self.cycle != 0
            && self.node_count > 0
            && self.contract_hash != 0
            && self.contract_hash
                == expected_wave_contract_hash(
                    self.wave_id,
                    self.parent_hash,
                    self.cycle,
                    self.node_count,
                    self.node_ids_hash,
                )
    }

    pub fn contract_hash(self) -> u64 {
        self.contract_hash
    }
}

/// Submitted by the parent scheduler after each child thread joins.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChildCompleteRecord {
    /// Matches `WaveRecord::wave_id` for the parent wave.
    pub wave_id: u64,
    /// FNV-1a hash of the child's plan node ID.
    pub node_id_hash: u64,
    /// 0 = completed normally, 1 = thread panicked.
    pub exit_status: u8,
    /// Self-hash binding all fields.
    pub contract_hash: u64,
}

impl ChildCompleteRecord {
    pub fn new(wave_id: u64, node_id_hash: u64, panicked: bool) -> Self {
        let exit_status = u8::from(panicked);
        let contract_hash =
            expected_child_contract_hash(wave_id, node_id_hash, exit_status);
        Self {
            wave_id,
            node_id_hash,
            exit_status,
            contract_hash,
        }
    }

    pub fn is_contract_valid(self) -> bool {
        self.wave_id != 0
            && self.node_id_hash != 0
            && self.contract_hash != 0
            && self.contract_hash
                == expected_child_contract_hash(self.wave_id, self.node_id_hash, self.exit_status)
    }

    pub fn contract_hash(self) -> u64 {
        self.contract_hash
    }
}

fn expected_wave_contract_hash(
    wave_id: u64,
    parent_hash: u64,
    cycle: u64,
    node_count: u16,
    node_ids_hash: u64,
) -> u64 {
    let mut h = 0xda3e_39cb_97f2_9809u64;
    h = mix(h, WAVE_RECORD_SCHEMA_VERSION);
    h = mix(h, wave_id);
    h = mix(h, parent_hash);
    h = mix(h, cycle);
    h = mix(h, node_count as u64);
    h = mix(h, node_ids_hash);
    h.max(1)
}

fn expected_child_contract_hash(wave_id: u64, node_id_hash: u64, exit_status: u8) -> u64 {
    let mut h = 0x8b3f_2711_cef4_0001u64;
    h = mix(h, CHILD_COMPLETE_SCHEMA_VERSION);
    h = mix(h, wave_id);
    h = mix(h, node_id_hash);
    h = mix(h, exit_status as u64);
    h.max(1)
}
