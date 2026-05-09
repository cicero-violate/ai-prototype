//! Versioned policy capability.

pub mod store;

pub use self::store::{
    PolicyEntry, PolicyLookupReceipt, PolicyProofReceipt, PolicyStore, PolicyStoreError,
    POLICY_FEEDBACK_HASH, POLICY_PROMOTION_SOURCE_SEQ,
};
