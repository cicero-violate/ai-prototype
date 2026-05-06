//! Versioned policy capability.

pub mod store;

pub use self::store::{
    PolicyEntry, PolicyProofReceipt, PolicyStore, PolicyStoreError, POLICY_FEEDBACK_HASH,
    POLICY_PROMOTION_SOURCE_SEQ,
};
