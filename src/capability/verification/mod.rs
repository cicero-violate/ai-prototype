//! Verification capability.
//!
//! Verification performs semantic artifact checks outside the kernel and
//! submits only the lineage proof token into the runtime.

pub mod proof;
pub mod record;

pub use self::proof::{
    append_verification_proof_record_ndjson, decode_verification_proof_record_ndjson,
    encode_verification_proof_record_ndjson, load_verification_proof_records_ndjson,
    verify_verification_proof_record_bindings, verify_verification_proof_record_order_ndjson,
    verify_verification_proof_record_replay, verify_verification_proof_record_replay_ndjson,
    verify_verification_proof_records, verify_verification_proof_records_ndjson, ProofSubjectKind,
    VerificationProofBinding, VerificationProofError, VerificationProofRecord,
    PROOF_FLAGS_REQUIRED, PROOF_FLAG_PHASE_VERIFIED, PROOF_FLAG_PROVENANCE_VERIFIED,
    PROOF_FLAG_RECEIPT_VERIFIED, PROOF_FLAG_TAMPER_REJECTED, VERIFICATION_PROOF_RECORD,
    VERIFICATION_PROOF_SCHEMA_VERSION,
};

pub use self::record::{
    ArtifactSemanticProfile, DeterministicSemanticVerifier, SemanticVerificationReceipt,
    VerificationCheck, VerificationDecision, VerificationRecord, VerificationRequest,
};