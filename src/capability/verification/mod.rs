//! Verification capability.
//!
//! Verification performs semantic artifact checks outside the kernel and
//! submits only the lineage proof token into the runtime.

pub mod record;

pub use self::record::{
    append_verification_proof_record_ndjson, decode_verification_proof_record_ndjson,
    encode_verification_proof_record_ndjson, load_verification_proof_records_ndjson,
    verify_verification_proof_record_bindings, verify_verification_proof_records,
    verify_verification_proof_records_ndjson,
    ArtifactSemanticProfile, DeterministicSemanticVerifier, ProofSubjectKind,
    SemanticVerificationReceipt, VerificationCheck, VerificationDecision, VerificationProofError,
    VerificationProofBinding, VerificationProofRecord, VerificationRecord, VerificationRequest,
    PROOF_FLAGS_REQUIRED, PROOF_FLAG_PHASE_VERIFIED, PROOF_FLAG_PROVENANCE_VERIFIED,
    PROOF_FLAG_RECEIPT_VERIFIED, PROOF_FLAG_TAMPER_REJECTED, VERIFICATION_PROOF_RECORD,
    VERIFICATION_PROOF_SCHEMA_VERSION,
};