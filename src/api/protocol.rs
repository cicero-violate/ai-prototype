//! External command/evidence protocol.

use crate::capability::EvidenceSubmission;
use crate::kernel::ControlEvent;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    SubmitEvidence(EvidenceSubmission),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControlEventResponse {
    pub event: ControlEvent,
}

