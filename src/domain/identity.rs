//! Deterministic local identity helpers for domain records.
//!
//! This intentionally avoids external dependencies and is not a cryptographic
//! proof. Capability/verification layers should own cryptographic receipts.

use std::fmt;

use serde_json::Value;

const DOMAIN_HASH_PREFIX: &str = "domain:";

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DomainHash(String);

impl DomainHash {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn validate(value: &str) -> Result<(), DomainHashError> {
        if value.trim().is_empty() {
            return Err(DomainHashError::Empty);
        }

        if value != value.trim() {
            return Err(DomainHashError::SurroundingWhitespace);
        }

        let Some(suffix) = value.strip_prefix(DOMAIN_HASH_PREFIX) else {
            return Err(DomainHashError::MissingDomainPrefix);
        };

        if suffix.is_empty() {
            return Err(DomainHashError::EmptyHashSuffix);
        }

        Ok(())
    }
}

impl AsRef<str> for DomainHash {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for DomainHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<String> for DomainHash {
    type Error = DomainHashError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::validate(&value)?;
        Ok(Self(value))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomainHashError {
    Empty,
    SurroundingWhitespace,
    MissingDomainPrefix,
    EmptyHashSuffix,
}

impl fmt::Display for DomainHashError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("domain hash must not be empty"),
            Self::SurroundingWhitespace => {
                f.write_str("domain hash must not contain surrounding whitespace")
            }
            Self::MissingDomainPrefix => f.write_str("domain hash must start with domain:"),
            Self::EmptyHashSuffix => f.write_str("domain hash suffix must not be empty"),
        }
    }
}

impl std::error::Error for DomainHashError {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DomainHashInput<'a> {
    Json(&'a Value),
    Parts(&'a [&'a str]),
}

impl<'a> DomainHashInput<'a> {
    pub fn json(record: &'a Value) -> Self {
        Self::Json(record)
    }

    pub fn parts(parts: &'a [&'a str]) -> Self {
        Self::Parts(parts)
    }
}

impl<'a> From<&'a Value> for DomainHashInput<'a> {
    fn from(record: &'a Value) -> Self {
        Self::Json(record)
    }
}

impl<'a> From<&'a [&'a str]> for DomainHashInput<'a> {
    fn from(parts: &'a [&'a str]) -> Self {
        Self::Parts(parts)
    }
}

impl<'a, const N: usize> From<&'a [&'a str; N]> for DomainHashInput<'a> {
    fn from(parts: &'a [&'a str; N]) -> Self {
        Self::Parts(&parts[..])
    }
}

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
    format!("{DOMAIN_HASH_PREFIX}{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_identity_is_order_sensitive_and_repeatable() {
        assert_eq!(stable_domain_id(&["a", "b"]), stable_domain_id(&["a", "b"]));
        assert_ne!(stable_domain_id(&["a", "b"]), stable_domain_id(&["b", "a"]));
    }

    #[test]
    fn domain_hash_newtype_validates_prefix_and_non_empty_suffix() {
        let hash = DomainHash::try_from("domain:abc123".to_string()).unwrap();

        assert_eq!(hash.as_str(), "domain:abc123");
        assert_eq!(hash.as_ref(), "domain:abc123");
        assert_eq!(hash.to_string(), "domain:abc123");
        assert_eq!(DomainHash::try_from("".to_string()), Err(DomainHashError::Empty));
        assert_eq!(
            DomainHash::try_from(" domain:abc123".to_string()),
            Err(DomainHashError::SurroundingWhitespace)
        );
        assert_eq!(
            DomainHash::try_from("hash:abc123".to_string()),
            Err(DomainHashError::MissingDomainPrefix)
        );
        assert_eq!(
            DomainHash::try_from("domain:".to_string()),
            Err(DomainHashError::EmptyHashSuffix)
        );
    }

    #[test]
    fn domain_hash_input_converts_json_and_parts() {
        let record = serde_json::json!({"kind":"signal","schema_version":1});
        assert_eq!(DomainHashInput::from(&record), DomainHashInput::Json(&record));
        assert_eq!(DomainHashInput::json(&record), DomainHashInput::Json(&record));

        let parts = ["schema:1", "kind:signal", "id:alpha"];
        assert_eq!(DomainHashInput::from(&parts), DomainHashInput::Parts(&parts));
        assert_eq!(DomainHashInput::parts(&parts), DomainHashInput::Parts(&parts));
    }
}
