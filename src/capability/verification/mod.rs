//! Verification capability.
//!
//! Verification performs semantic artifact checks outside the kernel and
//! submits only the lineage proof token into the runtime.

pub mod record;

pub use self::record::{
    ArtifactSemanticProfile, DeterministicSemanticVerifier, SemanticVerificationReceipt,
    VerificationCheck, VerificationDecision, VerificationRecord, VerificationRequest,
};