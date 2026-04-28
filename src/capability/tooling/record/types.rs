use crate::kernel::mix;

pub const TOOL_EFFECT_RECEIPT_SCHEMA_VERSION: u64 = 1;
pub const TOOL_EFFECT_RECEIPT_RECORD: u64 = 1;
pub const SANDBOX_PROCESS_RECEIPT_SCHEMA_VERSION: u64 = 1;
pub const SANDBOX_PROCESS_RECEIPT_RECORD: u64 = 2;
pub const PROCESS_EFFECT_RECEIPT_SCHEMA_VERSION: u64 = 1;
pub const PROCESS_EFFECT_RECEIPT_RECORD: u64 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolDecision {
    Succeeded,
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ToolKind {
    DeterministicArtifact = 1,
    Noop = 2,
    Denied = 3,
    SandboxFile = 4,
    SandboxProcess = 5,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ToolEffectKind {
    None = 0,
    Artifact = 1,
    Process = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Effect {
    pub kind: ToolEffectKind,
    pub digest: u64,
    pub metadata: u64,
}

impl Effect {
    pub fn none(digest: u64, metadata: u64) -> Self {
        Self {
            kind: ToolEffectKind::None,
            digest: digest.max(1),
            metadata: metadata.max(1),
        }
    }

    pub fn artifact(
        digest: u64,
        artifact_path_hash: u64,
        artifact_content_hash: u64,
        artifact_bytes: u64,
        sandbox_root_hash: u64,
    ) -> Self {
        let mut metadata = 0xa409_3822_299f_31d0u64;
        metadata = mix(metadata, artifact_path_hash);
        metadata = mix(metadata, artifact_content_hash);
        metadata = mix(metadata, artifact_bytes);
        metadata = mix(metadata, sandbox_root_hash);
        Self {
            kind: ToolEffectKind::Artifact,
            digest: digest.max(1),
            metadata: metadata.max(1),
        }
    }

    pub fn process(
        stdout_hash: u64,
        stderr_hash: u64,
        stdout_bytes: u64,
        stderr_bytes: u64,
        exit_status: u64,
        timed_out: bool,
    ) -> Self {
        let mut digest = 0x082e_fa98_ec4e_6c89u64;
        digest = mix(digest, stdout_hash);
        digest = mix(digest, stderr_hash);

        let mut metadata = 0x4528_21e6_38d0_1377u64;
        metadata = mix(metadata, stdout_bytes);
        metadata = mix(metadata, stderr_bytes);
        metadata = mix(metadata, exit_status);
        metadata = mix(metadata, timed_out as u64);

        Self {
            kind: ToolEffectKind::Process,
            digest: digest.max(1),
            metadata: metadata.max(1),
        }
    }

    pub fn is_valid(self) -> bool {
        self.digest != 0 && self.metadata != 0
    }

    pub fn contract_hash(self) -> u64 {
        let mut h = 0xd131_0ba6_981d_bacau64;
        h = mix(h, self.kind as u64);
        h = mix(h, self.digest);
        h = mix(h, self.metadata);
        h.max(1)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolSandboxError {
    SandboxIo,
    PathEscapesSandbox,
    ArtifactTooLarge,
    InvalidToolReceipt,
    InvalidToolReceiptRecord,
    InvalidReplay,
    CommandDenied,
    InvalidCommand,
    InvalidEnvironment,
    ProcessTimeout,
    OutputTooLarge,
}