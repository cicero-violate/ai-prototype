//! Learning capability.

pub mod promote;

pub use self::promote::{
    export_verified_distillation_row, DistillationExportError, DistillationExportInput,
    DistillationExportReceipt, DistillationRow, PolicyPromotion, DISTILLATION_ROW_RECORD,
    DISTILLATION_ROW_SCHEMA_VERSION,
};
pub use crate::capability::policy::{POLICY_FEEDBACK_HASH, POLICY_PROMOTION_SOURCE_SEQ};
