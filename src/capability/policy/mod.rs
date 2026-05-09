//! Versioned policy capability.

pub mod store;

pub use self::store::{
    POLICY_FEEDBACK_HASH, POLICY_PROMOTION_SOURCE_SEQ, PolicyEntry, PolicyLookupReceipt,
    PolicyProofReceipt, PolicyStore, PolicyStoreError,
};
