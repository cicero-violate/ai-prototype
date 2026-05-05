pub mod runtime;

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

pub fn stable_hash_bytes(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

pub fn stable_hash_fields(fields: &[&[u8]]) -> u64 {
    let mut hash = FNV_OFFSET;
    for field in fields {
        hash ^= field.len() as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
        for byte in *field {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::{stable_hash_bytes, stable_hash_fields};

    #[test]
    fn stable_hash_is_order_and_boundary_sensitive() {
        assert_eq!(stable_hash_bytes(b"trace"), stable_hash_bytes(b"trace"));
        assert_ne!(stable_hash_bytes(b"trace"), stable_hash_bytes(b"trace2"));
        assert_ne!(stable_hash_fields(&[b"ab", b"c"]), stable_hash_fields(&[b"a", b"bc"]));
    }
}