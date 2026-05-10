//! Deterministic local identity helpers for domain records.
//!
//! This intentionally avoids external dependencies and is not a cryptographic
//! proof. Capability/verification layers should own cryptographic receipts.

pub fn stable_domain_id(parts: &[&str]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for part in parts {
        for byte in part.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash ^= 0xff;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("domain:{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_identity_is_order_sensitive_and_repeatable() {
        assert_eq!(stable_domain_id(&["a", "b"]), stable_domain_id(&["a", "b"]));
        assert_ne!(stable_domain_id(&["a", "b"]), stable_domain_id(&["b", "a"]));
    }
}
