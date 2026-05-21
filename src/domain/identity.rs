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
    domain_hash_parts(parts).to_string()
}

pub fn domain_hash_parts(parts: &[&str]) -> DomainHash {
    let mut hash = 0xcbf29ce484222325u64;
    for part in parts {
        hash = write_hash_bytes(hash, part.as_bytes());
        hash ^= 0xff;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    DomainHash(format!("{DOMAIN_HASH_PREFIX}{hash:016x}"))
}

pub fn domain_hash_json(record: &Value) -> DomainHash {
    let hash = write_hash_bytes(0xcbf29ce484222325u64, &canonical_json_bytes(record));
    DomainHash(format!("{DOMAIN_HASH_PREFIX}{hash:016x}"))
}

fn write_hash_bytes(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

pub fn canonical_json_bytes(record: &Value) -> Vec<u8> {
    let mut out = Vec::new();
    write_canonical_json(record, &mut out);
    out
}

fn write_canonical_json(record: &Value, out: &mut Vec<u8>) {
    match record {
        Value::Null => out.extend_from_slice(b"null"),
        Value::Bool(false) => out.extend_from_slice(b"false"),
        Value::Bool(true) => out.extend_from_slice(b"true"),
        Value::Number(number) => out.extend_from_slice(number.to_string().as_bytes()),
        Value::String(value) => write_canonical_string(value, out),
        Value::Array(values) => {
            out.push(b'[');
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    out.push(b',');
                }
                write_canonical_json(value, out);
            }
            out.push(b']');
        }
        Value::Object(values) => {
            out.push(b'{');
            let mut entries: Vec<_> = values.iter().collect();
            entries.sort_by_key(|(left_key, _)| *left_key);
            for (index, (key, value)) in entries.into_iter().enumerate() {
                if index > 0 {
                    out.push(b',');
                }
                write_canonical_string(key, out);
                out.push(b':');
                write_canonical_json(value, out);
            }
            out.push(b'}');
        }
    }
}

fn write_canonical_string(value: &str, out: &mut Vec<u8>) {
    serde_json::to_writer(out, value).expect("serializing a JSON string into a Vec must not fail");
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
        let hash =
            DomainHash::try_from("domain:abc123".to_string()).expect("test setup should succeed");

        assert_eq!(hash.as_str(), "domain:abc123");
        assert_eq!(hash.as_ref(), "domain:abc123");
        assert_eq!(hash.to_string(), "domain:abc123");
        assert_eq!(
            DomainHash::try_from("".to_string()),
            Err(DomainHashError::Empty)
        );
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
        assert_eq!(
            DomainHashInput::from(&record),
            DomainHashInput::Json(&record)
        );
        assert_eq!(
            DomainHashInput::json(&record),
            DomainHashInput::Json(&record)
        );

        let parts = ["schema:1", "kind:signal", "id:alpha"];
        assert_eq!(
            DomainHashInput::from(&parts),
            DomainHashInput::Parts(&parts)
        );
        assert_eq!(
            DomainHashInput::parts(&parts),
            DomainHashInput::Parts(&parts)
        );
    }

    #[test]
    fn domain_hash_is_stable() {
        let parts = ["schema:1", "kind:signal", "id:alpha"];
        let record = serde_json::json!({
            "domain_id": "alpha",
            "kind": "signal",
            "schema_version": 1,
        });

        assert_eq!(domain_hash_parts(&parts), domain_hash_parts(&parts));
        assert_eq!(
            domain_hash_parts(&parts).to_string(),
            stable_domain_id(&parts)
        );
        assert_eq!(domain_hash_json(&record), domain_hash_json(&record));
    }

    #[test]
    fn domain_hash_changes_when_material_field_changes() {
        let original = serde_json::json!({
            "domain_id": "alpha",
            "kind": "signal",
            "schema_version": 1,
            "source_hash": "domain:source-a",
        });
        let changed = serde_json::json!({
            "domain_id": "alpha",
            "kind": "signal",
            "schema_version": 1,
            "source_hash": "domain:source-b",
        });

        assert_ne!(domain_hash_json(&original), domain_hash_json(&changed));
    }

    #[test]
    fn domain_hash_changes_when_schema_version_changes() {
        let original = serde_json::json!({
            "domain_id": "alpha",
            "kind": "signal",
            "schema_version": 1,
            "source_hash": "domain:source-a",
        });
        let changed = serde_json::json!({
            "domain_id": "alpha",
            "kind": "signal",
            "schema_version": 2,
            "source_hash": "domain:source-a",
        });

        assert_ne!(domain_hash_json(&original), domain_hash_json(&changed));
    }
}
