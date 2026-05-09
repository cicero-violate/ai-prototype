//! Learning capability.

pub mod promote;

pub use self::promote::{
    DISTILLATION_ROW_RECORD, DISTILLATION_ROW_SCHEMA_VERSION, DistillationExportError,
    DistillationExportInput, DistillationExportReceipt, DistillationRow, PolicyPromotion,
    export_verified_distillation_row,
};
pub use crate::capability::policy::{POLICY_FEEDBACK_HASH, POLICY_PROMOTION_SOURCE_SEQ};
