#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CapabilityRegistryProjection {
    pub route_count: u64,
    pub policy_hash: u64,
}

impl CapabilityRegistryProjection {
    pub const fn none() -> Self {
        Self {
            route_count: 0,
            policy_hash: 0,
        }
    }

    pub const fn new(route_count: u64, policy_hash: u64) -> Self {
        Self {
            route_count,
            policy_hash,
        }
    }

    pub const fn is_empty(self) -> bool {
        self.route_count == 0 && self.policy_hash == 0
    }

    pub const fn is_valid(self) -> bool {
        self.is_empty() || self.policy_hash != 0
    }
}
