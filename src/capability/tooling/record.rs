//! Durable tool execution payload owned by the tooling capability.
//!
//! Tooling is the first capability boundary that must stop being a synthetic
//! record facade. This module now models the live execution loop:
//!
//! ```text
//! request -> authorize -> execute -> receipt -> tlog
//! ```
//!
//! The kernel still sees only evidence, gates, packet effects, and hashes.
//! External work is isolated here behind deny-by-default capability routing and
//! an append-only receipt sidecar.

use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::capability::{
    CapabilityId, CapabilityRegistry, EvidenceProducer, EvidenceSubmission, PacketEffect,
};
use crate::kernel::{
    mix, Cause, ControlEvent, Decision, EventKind, Evidence, GateId, GateStatus, Packet, Phase,
    TLog,
};

pub const TOOL_EFFECT_RECEIPT_SCHEMA_VERSION: u64 = 1;
pub const TOOL_EFFECT_RECEIPT_RECORD: u64 = 1;
pub const SANDBOX_PROCESS_RECEIPT_SCHEMA_VERSION: u64 = 1;
pub const SANDBOX_PROCESS_RECEIPT_RECORD: u64 = 2;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToolRequest {
    pub capability: CapabilityId,
    pub registry_policy_hash: u64,
    pub tool_kind: ToolKind,
    pub objective_id: u64,
    pub task_id: u64,
    pub command_hash: u64,
    pub input_hash: u64,
    pub requested_effect: PacketEffect,
}

impl ToolRequest {
    pub fn from_packet(packet: Packet) -> Self {
        Self::from_packet_with_registry(packet, CapabilityRegistry::canonical())
    }

    pub fn from_packet_with_registry(packet: Packet, registry: CapabilityRegistry) -> Self {
        Self::from_packet_with_registry_and_kind(packet, registry, ToolKind::DeterministicArtifact)
    }

    pub fn from_packet_with_registry_and_kind(
        packet: Packet,
        registry: CapabilityRegistry,
        tool_kind: ToolKind,
    ) -> Self {
        Self {
            capability: CapabilityId::Tooling,
            registry_policy_hash: registry.policy_hash(),
            tool_kind: if packet.has_ready_task() {
                tool_kind
            } else {
                ToolKind::Denied
            },
            objective_id: packet.objective_id,
            task_id: packet.active_task_id,
            command_hash: tool_command_hash(packet),
            input_hash: tool_input_hash(packet),
            requested_effect: PacketEffect::MaterializeArtifact,
        }
    }

    pub fn is_admissible(self) -> bool {
        self.capability == CapabilityId::Tooling
            && self.registry_policy_hash != 0
            && matches!(
                self.tool_kind,
                ToolKind::DeterministicArtifact | ToolKind::SandboxFile
            )
            && self.objective_id != 0
            && self.task_id != 0
            && self.command_hash != 0
            && self.input_hash != 0
            && self.requested_effect == PacketEffect::MaterializeArtifact
    }

    pub fn matches_packet(self, packet: Packet) -> bool {
        self.objective_id == packet.objective_id
            && self.task_id == packet.active_task_id
            && self.command_hash == tool_command_hash(packet)
            && self.input_hash == tool_input_hash(packet)
            && self.requested_effect == PacketEffect::MaterializeArtifact
    }

    pub fn contract_hash(self) -> u64 {
        let mut h = 0xbb67ae8584caa73bu64;
        h = mix(h, self.capability as u64);
        h = mix(h, self.registry_policy_hash);
        h = mix(h, self.tool_kind as u64);
        h = mix(h, self.objective_id);
        h = mix(h, self.task_id);
        h = mix(h, self.command_hash);
        h = mix(h, self.input_hash);
        h = mix(h, self.requested_effect as u64);
        h.max(1)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToolReceipt {
    pub registry_policy_hash: u64,
    pub objective_id: u64,
    pub task_id: u64,
    pub command_hash: u64,
    pub input_hash: u64,
    pub output_hash: u64,
    pub effect: Effect,
    pub exit_code: u8,
    pub artifact_path_hash: u64,
    pub artifact_content_hash: u64,
    pub artifact_bytes: u64,
    pub sandbox_root_hash: u64,
    pub receipt_hash: u64,
}

impl ToolReceipt {
    pub fn is_success_for(self, request: ToolRequest) -> bool {
        self.exit_code == 0
            && self.registry_policy_hash == request.registry_policy_hash
            && self.objective_id == request.objective_id
            && self.task_id == request.task_id
            && self.command_hash == request.command_hash
            && self.input_hash == request.input_hash
            && self.output_hash != 0
            && self.effect
                == Effect::artifact(
                    self.output_hash,
                    self.artifact_path_hash,
                    self.artifact_content_hash,
                    self.artifact_bytes,
                    self.sandbox_root_hash,
                )
            && self.receipt_hash
                == expected_receipt_hash_for_effect(
                    request,
                    self.exit_code,
                    self.effect,
                )
    }

    pub fn is_sandbox_artifact_bound(self) -> bool {
        self.effect.kind == ToolEffectKind::Artifact
            && self.artifact_path_hash != 0
            && self.artifact_content_hash != 0
            && self.artifact_bytes != 0
            && self.sandbox_root_hash != 0
    }

    fn success(request: ToolRequest, output_hash: u64) -> Self {
        Self::success_with_artifact(request, output_hash, 0, 0, 0, 0)
    }

    fn success_with_artifact(
        request: ToolRequest,
        output_hash: u64,
        artifact_path_hash: u64,
        artifact_content_hash: u64,
        artifact_bytes: u64,
        sandbox_root_hash: u64,
    ) -> Self {
        let effect = Effect::artifact(
            output_hash,
            artifact_path_hash,
            artifact_content_hash,
            artifact_bytes,
            sandbox_root_hash,
        );
        Self {
            registry_policy_hash: request.registry_policy_hash,
            objective_id: request.objective_id,
            task_id: request.task_id,
            command_hash: request.command_hash,
            input_hash: request.input_hash,
            output_hash,
            effect,
            exit_code: 0,
            artifact_path_hash,
            artifact_content_hash,
            artifact_bytes,
            sandbox_root_hash,
            receipt_hash: expected_receipt_hash_for_effect(request, 0, effect),
        }
    }

    fn failure(request: ToolRequest, exit_code: u8) -> Self {
        let output_hash = tool_failure_hash(request, exit_code);
        let effect = Effect::none(output_hash, exit_code as u64);
        Self {
            registry_policy_hash: request.registry_policy_hash,
            objective_id: request.objective_id,
            task_id: request.task_id,
            command_hash: request.command_hash,
            input_hash: request.input_hash,
            output_hash,
            effect,
            exit_code,
            artifact_path_hash: 0,
            artifact_content_hash: 0,
            artifact_bytes: 0,
            sandbox_root_hash: 0,
            receipt_hash: expected_receipt_hash_for_effect(request, exit_code, effect),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeterministicToolExecutor {
    pub max_input_hash: u64,
    pub registry: CapabilityRegistry,
}

impl Default for DeterministicToolExecutor {
    fn default() -> Self {
        Self {
            max_input_hash: u64::MAX,
            registry: CapabilityRegistry::canonical(),
        }
    }
}

impl DeterministicToolExecutor {
    pub fn execute(self, request: ToolRequest) -> ToolReceipt {
        if !self.authorizes(request) || request.input_hash > self.max_input_hash {
            return ToolReceipt::failure(request, 126);
        }

        ToolReceipt::success(request, tool_output_hash(request))
    }

    pub fn execute_packet(self, packet: Packet) -> ToolReceipt {
        let request = ToolRequest::from_packet_with_registry(packet, self.registry);
        if !self.authorizes(request) || request.input_hash > self.max_input_hash {
            return ToolReceipt::failure(request, 126);
        }

        let mut after = packet;
        after.materialize_artifact();
        ToolReceipt::success(request, tool_effect_output_hash(packet, after))
    }

    pub fn authorizes(self, request: ToolRequest) -> bool {
        request.is_admissible()
            && request.registry_policy_hash == self.registry.policy_hash()
            && self.registry.permits_effect(
                request.capability,
                GateId::Execution,
                Evidence::ArtifactReceipt,
                request.requested_effect,
            )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveSandboxToolExecutor {
    pub root: PathBuf,
    pub max_artifact_bytes: u64,
    pub registry: CapabilityRegistry,
}

impl LiveSandboxToolExecutor {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            max_artifact_bytes: 64 * 1024,
            registry: CapabilityRegistry::canonical(),
        }
    }

    pub fn with_registry(mut self, registry: CapabilityRegistry) -> Self {
        self.registry = registry;
        self
    }

    pub fn with_max_artifact_bytes(mut self, max_artifact_bytes: u64) -> Self {
        self.max_artifact_bytes = max_artifact_bytes;
        self
    }

    pub fn execute_packet(&self, packet: Packet) -> Result<ToolExecutionRecord, ToolSandboxError> {
        let request = self.packet_request(packet);
        let request = self.authorize_request(request)?;
        let effect = self.execute_artifact(request, packet)?;
        let receipt = self.artifact_receipt(&effect);
        Ok(ToolExecutionRecord {
            request: effect.request,
            receipt,
        })
    }

    fn packet_request(&self, packet: Packet) -> ToolRequest {
        ToolRequest::from_packet_with_registry_and_kind(
            packet,
            self.registry,
            ToolKind::SandboxFile,
        )
    }

    fn authorize_request(&self, request: ToolRequest) -> Result<ToolRequest, ToolSandboxError> {
        if self.authorizes(request) {
            Ok(request)
        } else {
            Err(ToolSandboxError::CommandDenied)
        }
    }

    fn execute_artifact(
        &self,
        request: ToolRequest,
        before: Packet,
    ) -> Result<SandboxArtifactEffect, ToolSandboxError> {
        let mut after = before;
        after.materialize_artifact();
        let body = sandbox_artifact_body(request, before, after);
        let artifact_bytes = body.len() as u64;
        if artifact_bytes == 0 || artifact_bytes > self.max_artifact_bytes {
            return Err(ToolSandboxError::ArtifactTooLarge);
        }

        let root = self.prepare_root()?;
        let artifact_path = self.artifact_path_for_root(&root, request)?;
        self.write_artifact(&artifact_path, &body)?;

        let artifact_path_hash = path_hash(&artifact_relative_name(request));
        let artifact_content_hash = bytes_hash(&body);
        let sandbox_root_hash = path_hash(root.to_string_lossy().as_bytes());
        let effect = Effect::artifact(
            tool_effect_output_hash(before, after),
            artifact_path_hash,
            artifact_content_hash,
            artifact_bytes,
            sandbox_root_hash,
        );

        Ok(SandboxArtifactEffect {
            request,
            artifact_path_hash,
            artifact_content_hash,
            artifact_bytes,
            sandbox_root_hash,
            effect,
        })
    }

    fn write_artifact(&self, artifact_path: &Path, body: &[u8]) -> Result<(), ToolSandboxError> {
        let parent = artifact_path.parent().ok_or(ToolSandboxError::SandboxIo)?;
        fs::create_dir_all(parent).map_err(|_| ToolSandboxError::SandboxIo)?;
        {
            let mut file = File::create(artifact_path).map_err(|_| ToolSandboxError::SandboxIo)?;
            file.write_all(body)
                .map_err(|_| ToolSandboxError::SandboxIo)?;
            file.sync_all().map_err(|_| ToolSandboxError::SandboxIo)?;
        }
        sync_dir(parent)
    }

    fn artifact_receipt(&self, effect: &SandboxArtifactEffect) -> ToolReceipt {
        let receipt = ToolReceipt::success_with_artifact(
            effect.request,
            effect.effect.digest,
            effect.artifact_path_hash,
            effect.artifact_content_hash,
            effect.artifact_bytes,
            effect.sandbox_root_hash,
        );
        debug_assert_eq!(receipt.effect, effect.effect);
        receipt
    }

    pub fn artifact_path_for(&self, request: ToolRequest) -> Result<PathBuf, ToolSandboxError> {
        let root = self.prepare_root()?;
        self.artifact_path_for_root(&root, request)
    }

    fn artifact_path_for_root(
        &self,
        root: &Path,
        request: ToolRequest,
    ) -> Result<PathBuf, ToolSandboxError> {
        let path = root
            .join("artifacts")
            .join(artifact_relative_name(request));
        ensure_relative_name(&artifact_relative_name(request))?;
        ensure_under(root, &path)?;
        Ok(path)
    }

    pub fn authorizes(&self, request: ToolRequest) -> bool {
        request.is_admissible()
            && request.tool_kind == ToolKind::SandboxFile
            && request.registry_policy_hash == self.registry.policy_hash()
            && self.registry.permits_effect(
                request.capability,
                GateId::Execution,
                Evidence::ArtifactReceipt,
                request.requested_effect,
            )
    }

    fn prepare_root(&self) -> Result<PathBuf, ToolSandboxError> {
        fs::create_dir_all(&self.root).map_err(|_| ToolSandboxError::SandboxIo)?;
        self.root
            .canonicalize()
            .map_err(|_| ToolSandboxError::SandboxIo)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SandboxArtifactEffect {
    request: ToolRequest,
    artifact_path_hash: u64,
    artifact_content_hash: u64,
    artifact_bytes: u64,
    sandbox_root_hash: u64,
    effect: Effect,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolExecutionRecord {
    pub request: ToolRequest,
    pub receipt: ToolReceipt,
}

impl ToolExecutionRecord {
    pub fn from_packet(packet: Packet) -> Self {
        let request = ToolRequest::from_packet(packet);
        let receipt = DeterministicToolExecutor::default().execute_packet(packet);
        Self { request, receipt }
    }

    pub fn from_packet_with_executor(packet: Packet, executor: DeterministicToolExecutor) -> Self {
        let request = ToolRequest::from_packet_with_registry(packet, executor.registry);
        let receipt = executor.execute_packet(packet);
        Self { request, receipt }
    }

    pub fn from_request(request: ToolRequest) -> Self {
        let receipt = DeterministicToolExecutor::default().execute(request);
        Self { request, receipt }
    }

    pub fn decision(&self) -> ToolDecision {
        if self.is_valid() {
            ToolDecision::Succeeded
        } else {
            ToolDecision::Failed
        }
    }

    pub fn is_valid(&self) -> bool {
        self.request.is_admissible()
            && self.receipt.is_success_for(self.request)
            && self.request.requested_effect == PacketEffect::MaterializeArtifact
    }

    pub fn verifies_persisted_event(&self, event: &ControlEvent) -> bool {
        persisted_execution_effect_is_valid(self, event)
    }

    pub fn effect_receipt_for_event(&self, event: &ControlEvent) -> Option<ToolEffectReceipt> {
        ToolEffectReceipt::from_persisted_event(self, event)
    }

    pub fn submission(&self) -> EvidenceSubmission {
        let passed = self.decision() == ToolDecision::Succeeded;
        EvidenceSubmission::with_effect_payload(
            GateId::Execution,
            Evidence::ArtifactReceipt,
            passed,
            if passed {
                PacketEffect::MaterializeArtifact
            } else {
                PacketEffect::None
            },
            tooling_payload_hash(self),
        )
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SandboxProcessRequest {
    pub capability: CapabilityId,
    pub registry_policy_hash: u64,
    pub command_hash: u64,
    pub argv_hash: u64,
    pub cwd_hash: u64,
    pub env_hash: u64,
    pub timeout_ms: u64,
    pub max_output_bytes: u64,
}

impl SandboxProcessRequest {
    pub fn contract_hash(&self) -> u64 {
        let mut h = 0x510e_527f_ade6_82d1u64;
        h = mix(h, self.capability as u64);
        h = mix(h, self.registry_policy_hash);
        h = mix(h, self.command_hash);
        h = mix(h, self.argv_hash);
        h = mix(h, self.cwd_hash);
        h = mix(h, self.env_hash);
        h = mix(h, self.timeout_ms);
        h = mix(h, self.max_output_bytes);
        h.max(1)
    }

    pub fn is_admissible(&self) -> bool {
        self.capability == CapabilityId::Tooling
            && self.registry_policy_hash != 0
            && self.command_hash != 0
            && self.argv_hash != 0
            && self.cwd_hash != 0
            && self.env_hash != 0
            && self.timeout_ms != 0
            && self.max_output_bytes != 0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SandboxProcessReceipt {
    pub request_hash: u64,
    pub registry_policy_hash: u64,
    pub command_hash: u64,
    pub argv_hash: u64,
    pub cwd_hash: u64,
    pub env_hash: u64,
    pub timeout_ms: u64,
    pub max_output_bytes: u64,
    pub effect: Effect,
    pub stdout_hash: u64,
    pub stderr_hash: u64,
    pub stdout_bytes: u64,
    pub stderr_bytes: u64,
    pub exit_status: u64,
    pub timed_out: bool,
    pub receipt_hash: u64,
}

impl SandboxProcessReceipt {
    pub fn effect_is_normalized(&self) -> bool {
        self.effect
            == Effect::process(
                self.stdout_hash,
                self.stderr_hash,
                self.stdout_bytes,
                self.stderr_bytes,
                self.exit_status,
                self.timed_out,
            )
    }

    pub fn is_valid_for(&self, request: &SandboxProcessRequest) -> bool {
        request.is_admissible()
            && self.request_hash == request.contract_hash()
            && self.registry_policy_hash == request.registry_policy_hash
            && self.command_hash == request.command_hash
            && self.argv_hash == request.argv_hash
            && self.cwd_hash == request.cwd_hash
            && self.env_hash == request.env_hash
            && self.timeout_ms == request.timeout_ms
            && self.max_output_bytes == request.max_output_bytes
            && self.effect_is_normalized()
            && self.stdout_hash != 0
            && self.stderr_hash != 0
            && self.stdout_bytes <= self.max_output_bytes
            && self.stderr_bytes <= self.max_output_bytes
            && self.receipt_hash == expected_process_receipt_hash(self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveSandboxProcessExecutor {
    pub root: PathBuf,
    pub allowed_commands: Vec<String>,
    pub locked_env: Vec<(String, String)>,
    pub timeout_ms: u64,
    pub max_output_bytes: u64,
    pub registry: CapabilityRegistry,
}

impl LiveSandboxProcessExecutor {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            allowed_commands: Vec::new(),
            locked_env: Vec::new(),
            timeout_ms: 1000,
            max_output_bytes: 64 * 1024,
            registry: CapabilityRegistry::canonical(),
        }
    }

    pub fn with_allowed_command(mut self, command: impl Into<String>) -> Self {
        self.allowed_commands.push(command.into());
        self
    }

    pub fn with_locked_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.locked_env.push((key.into(), value.into()));
        self
    }

    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    pub fn with_max_output_bytes(mut self, max_output_bytes: u64) -> Self {
        self.max_output_bytes = max_output_bytes;
        self
    }

    pub fn with_registry(mut self, registry: CapabilityRegistry) -> Self {
        self.registry = registry;
        self
    }

    pub fn execute_process(
        &self,
        command: &str,
        args: &[&str],
        cwd_relative: &str,
    ) -> Result<SandboxProcessReceipt, ToolSandboxError> {
        let request = self.process_plan(command, args, cwd_relative)?;
        let request = self.authorize_process(request)?;
        let effect = self.execute_authorized_process(&request)?;
        Ok(self.process_receipt(effect))
    }

    pub fn replay_receipt(
        &self,
        receipt: &SandboxProcessReceipt,
        command: &str,
        args: &[&str],
        cwd_relative: &str,
    ) -> Result<bool, ToolSandboxError> {
        let request = self.process_plan(command, args, cwd_relative)?;
        let request = self.authorize_process(request)?;
        Ok(receipt.is_valid_for(&request.request))
    }

    fn process_plan(
        &self,
        command: &str,
        args: &[&str],
        cwd_relative: &str,
    ) -> Result<SandboxProcessPlan, ToolSandboxError> {
        let root = self.prepare_root()?;
        self.validate_command(command)?;
        self.validate_args(args)?;
        self.validate_locked_env()?;
        let cwd = self.validated_cwd(&root, cwd_relative)?;
        let request = self.process_request(command, args, &cwd);
        let io = self.process_io_paths(&root, &request)?;
        Ok(SandboxProcessPlan {
            request,
            command: command.to_string(),
            args: args.iter().map(|arg| (*arg).to_string()).collect(),
            cwd,
            io,
        })
    }

    fn execute_authorized_process(
        &self,
        plan: &AuthorizedSandboxProcess,
    ) -> Result<SandboxProcessEffect, ToolSandboxError> {
        let stdout_file = File::create(&plan.io.stdout).map_err(|_| ToolSandboxError::SandboxIo)?;
        let stderr_file = File::create(&plan.io.stderr).map_err(|_| ToolSandboxError::SandboxIo)?;

        let mut child = Command::new(plan.command.as_str())
            .args(plan.args.iter().map(String::as_str))
            .current_dir(&plan.cwd)
            .env_clear()
            .envs(self.locked_env.iter().map(|(k, v)| (k.as_str(), v.as_str())))
            .stdin(Stdio::null())
            .stdout(Stdio::from(stdout_file))
            .stderr(Stdio::from(stderr_file))
            .spawn()
            .map_err(|_| ToolSandboxError::SandboxIo)?;

        let started = Instant::now();
        let timeout = Duration::from_millis(self.timeout_ms);
        let (exit_status, timed_out) = loop {
            if let Some(status) = child
                .try_wait()
                .map_err(|_| ToolSandboxError::SandboxIo)?
            {
                break (status.code().unwrap_or(255) as u64, false);
            }

            if started.elapsed() >= timeout {
                let _ = child.kill();
                let _ = child.wait();
                break (124, true);
            }

            thread::sleep(Duration::from_millis(10));
        };

        let stdout = bounded_file_bytes(&plan.io.stdout, self.max_output_bytes)?;
        let stderr = bounded_file_bytes(&plan.io.stderr, self.max_output_bytes)?;
        sync_dir(plan.io.stdout.parent().ok_or(ToolSandboxError::SandboxIo)?)?;
        let stdout_hash = bytes_hash(&stdout);
        let stderr_hash = bytes_hash(&stderr);
        let stdout_bytes = stdout.len() as u64;
        let stderr_bytes = stderr.len() as u64;
        let effect = Effect::process(
            stdout_hash,
            stderr_hash,
            stdout_bytes,
            stderr_bytes,
            exit_status,
            timed_out,
        );

        Ok(SandboxProcessEffect {
            request: plan.request,
            effect,
            stdout_hash,
            stderr_hash,
            stdout_bytes,
            stderr_bytes,
            exit_status,
            timed_out,
        })
    }

    fn process_receipt(&self, effect: SandboxProcessEffect) -> SandboxProcessReceipt {
        let mut receipt = SandboxProcessReceipt {
            request_hash: effect.request.contract_hash(),
            registry_policy_hash: effect.request.registry_policy_hash,
            command_hash: effect.request.command_hash,
            argv_hash: effect.request.argv_hash,
            cwd_hash: effect.request.cwd_hash,
            env_hash: effect.request.env_hash,
            timeout_ms: effect.request.timeout_ms,
            max_output_bytes: effect.request.max_output_bytes,
            effect: effect.effect,
            stdout_hash: effect.stdout_hash,
            stderr_hash: effect.stderr_hash,
            stdout_bytes: effect.stdout_bytes,
            stderr_bytes: effect.stderr_bytes,
            exit_status: effect.exit_status,
            timed_out: effect.timed_out,
            receipt_hash: 0,
        };
        receipt.receipt_hash = expected_process_receipt_hash(&receipt);
        receipt
    }

    fn prepare_root(&self) -> Result<PathBuf, ToolSandboxError> {
        fs::create_dir_all(&self.root).map_err(|_| ToolSandboxError::SandboxIo)?;
        self.root
            .canonicalize()
            .map_err(|_| ToolSandboxError::SandboxIo)
    }

    fn validate_command(&self, command: &str) -> Result<(), ToolSandboxError> {
        ensure_process_token(command)?;
        if self.allowed_commands.iter().any(|allowed| allowed == command) {
            Ok(())
        } else {
            Err(ToolSandboxError::CommandDenied)
        }
    }

    fn validate_args(&self, args: &[&str]) -> Result<(), ToolSandboxError> {
        if args.iter().any(|arg| arg.contains('\0')) {
            Err(ToolSandboxError::InvalidCommand)
        } else {
            Ok(())
        }
    }

    fn validate_locked_env(&self) -> Result<(), ToolSandboxError> {
        for (key, value) in &self.locked_env {
            if key.is_empty()
                || key.contains('=')
                || key.contains('\0')
                || value.contains('\0')
            {
                return Err(ToolSandboxError::InvalidEnvironment);
            }
        }
        Ok(())
    }

    fn validated_cwd(&self, root: &Path, cwd_relative: &str) -> Result<PathBuf, ToolSandboxError> {
        ensure_relative_process_path(cwd_relative)?;
        let cwd = root.join(cwd_relative);
        fs::create_dir_all(&cwd).map_err(|_| ToolSandboxError::SandboxIo)?;
        let canonical = cwd.canonicalize().map_err(|_| ToolSandboxError::SandboxIo)?;
        if canonical.starts_with(root) {
            Ok(canonical)
        } else {
            Err(ToolSandboxError::PathEscapesSandbox)
        }
    }

    fn process_request(&self, command: &str, args: &[&str], cwd: &Path) -> SandboxProcessRequest {
        SandboxProcessRequest {
            capability: CapabilityId::Tooling,
            registry_policy_hash: self.registry.policy_hash(),
            command_hash: string_hash(command),
            argv_hash: argv_hash(command, args),
            cwd_hash: path_hash(cwd.to_string_lossy().as_bytes()),
            env_hash: locked_env_hash(&self.locked_env),
            timeout_ms: self.timeout_ms,
            max_output_bytes: self.max_output_bytes,
        }
    }

    fn authorize_process(
        &self,
        plan: SandboxProcessPlan,
    ) -> Result<AuthorizedSandboxProcess, ToolSandboxError> {
        if plan.request.is_admissible()
            && plan.request.registry_policy_hash == self.registry.policy_hash()
            && self.registry.permits_effect(
                CapabilityId::Tooling,
                GateId::Execution,
                Evidence::ArtifactReceipt,
                PacketEffect::MaterializeArtifact,
            )
        {
            Ok(AuthorizedSandboxProcess {
                request: plan.request,
                command: plan.command,
                args: plan.args,
                cwd: plan.cwd,
                io: plan.io,
            })
        } else {
            Err(ToolSandboxError::CommandDenied)
        }
    }

    fn process_io_paths(
        &self,
        root: &Path,
        request: &SandboxProcessRequest,
    ) -> Result<ProcessIoPaths, ToolSandboxError> {
        let dir = root.join("process");
        fs::create_dir_all(&dir).map_err(|_| ToolSandboxError::SandboxIo)?;
        ensure_under(root, &dir.join("probe"))?;
        Ok(ProcessIoPaths {
            stdout: dir.join(format!("{:016x}.stdout", request.contract_hash())),
            stderr: dir.join(format!("{:016x}.stderr", request.contract_hash())),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ProcessIoPaths {
    stdout: PathBuf,
    stderr: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SandboxProcessPlan {
    request: SandboxProcessRequest,
    command: String,
    args: Vec<String>,
    cwd: PathBuf,
    io: ProcessIoPaths,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AuthorizedSandboxProcess {
    request: SandboxProcessRequest,
    command: String,
    args: Vec<String>,
    cwd: PathBuf,
    io: ProcessIoPaths,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SandboxProcessEffect {
    request: SandboxProcessRequest,
    effect: Effect,
    stdout_hash: u64,
    stderr_hash: u64,
    stdout_bytes: u64,
    stderr_bytes: u64,
    exit_status: u64,
    timed_out: bool,
}

pub fn append_sandbox_process_receipt_ndjson(
    path: impl AsRef<Path>,
    receipt: &SandboxProcessReceipt,
) -> Result<(), ToolSandboxError> {
    if !receipt.effect.is_valid()
        || !receipt.effect_is_normalized()
        || receipt.receipt_hash != expected_process_receipt_hash(receipt)
    {
        return Err(ToolSandboxError::InvalidToolReceipt);
    }

    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|_| ToolSandboxError::SandboxIo)?;
        }
    }

    {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|_| ToolSandboxError::SandboxIo)?;
        writeln!(file, "{}", encode_sandbox_process_receipt_ndjson(receipt))
            .map_err(|_| ToolSandboxError::SandboxIo)?;
        file.sync_all().map_err(|_| ToolSandboxError::SandboxIo)?;
    }

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            sync_dir(parent)?;
        }
    }

    Ok(())
}

pub fn load_sandbox_process_receipts_ndjson(
    path: impl AsRef<Path>,
) -> Result<Vec<SandboxProcessReceipt>, ToolSandboxError> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(path).map_err(|_| ToolSandboxError::SandboxIo)?;
    let reader = BufReader::new(file);
    let mut receipts = Vec::new();
    for line in reader.lines() {
        let line = line.map_err(|_| ToolSandboxError::SandboxIo)?;
        if !line.trim().is_empty() {
            receipts.push(decode_sandbox_process_receipt_ndjson(&line)?);
        }
    }
    Ok(receipts)
}

pub fn verify_sandbox_process_receipts(
    executor: &LiveSandboxProcessExecutor,
    receipts: &[SandboxProcessReceipt],
    command: &str,
    args: &[&str],
    cwd_relative: &str,
) -> Result<usize, ToolSandboxError> {
    for receipt in receipts {
        if !executor.replay_receipt(receipt, command, args, cwd_relative)? {
            return Err(ToolSandboxError::InvalidReplay);
        }
    }
    Ok(receipts.len())
}

pub fn encode_sandbox_process_receipt_ndjson(receipt: &SandboxProcessReceipt) -> String {
    let fields = [
        SANDBOX_PROCESS_RECEIPT_SCHEMA_VERSION,
        SANDBOX_PROCESS_RECEIPT_RECORD,
        receipt.request_hash,
        receipt.registry_policy_hash,
        receipt.command_hash,
        receipt.argv_hash,
        receipt.cwd_hash,
        receipt.env_hash,
        receipt.timeout_ms,
        receipt.max_output_bytes,
        receipt.effect.kind as u64,
        receipt.effect.digest,
        receipt.effect.metadata,
        receipt.stdout_hash,
        receipt.stderr_hash,
        receipt.stdout_bytes,
        receipt.stderr_bytes,
        receipt.exit_status,
        receipt.timed_out as u64,
        receipt.receipt_hash,
    ];
    let body = fields
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{body}]")
}

pub fn decode_sandbox_process_receipt_ndjson(
    line: &str,
) -> Result<SandboxProcessReceipt, ToolSandboxError> {
    let trimmed = line.trim();
    let body = trimmed
        .strip_prefix('[')
        .and_then(|v| v.strip_suffix(']'))
        .ok_or(ToolSandboxError::InvalidToolReceiptRecord)?;
    let fields = if body.trim().is_empty() {
        Vec::new()
    } else {
        body.split(',')
            .map(|raw| {
                raw.trim()
                    .parse::<u64>()
                    .map_err(|_| ToolSandboxError::InvalidToolReceiptRecord)
            })
            .collect::<Result<Vec<_>, _>>()?
    };

    if fields.len() != 20
        || fields[0] != SANDBOX_PROCESS_RECEIPT_SCHEMA_VERSION
        || fields[1] != SANDBOX_PROCESS_RECEIPT_RECORD
    {
        return Err(ToolSandboxError::InvalidToolReceiptRecord);
    }

    let effect = Effect {
        kind: tool_effect_kind_from_u64(fields[10])?,
        digest: fields[11],
        metadata: fields[12],
    };

    let receipt = SandboxProcessReceipt {
        request_hash: fields[2],
        registry_policy_hash: fields[3],
        command_hash: fields[4],
        argv_hash: fields[5],
        cwd_hash: fields[6],
        env_hash: fields[7],
        timeout_ms: fields[8],
        max_output_bytes: fields[9],
        effect,
        stdout_hash: fields[13],
        stderr_hash: fields[14],
        stdout_bytes: fields[15],
        stderr_bytes: fields[16],
        exit_status: fields[17],
        timed_out: fields[18] != 0,
        receipt_hash: fields[19],
    };

    if !receipt.effect.is_valid()
        || !receipt.effect_is_normalized()
        || receipt.receipt_hash != expected_process_receipt_hash(&receipt)
    {
        return Err(ToolSandboxError::InvalidToolReceipt);
    }

    Ok(receipt)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToolEffectReceipt {
    pub capability: CapabilityId,
    pub registry_policy_hash: u64,
    pub request_hash: u64,
    pub receipt_hash: u64,
    pub effect_hash: u64,
    pub effect: Effect,
    pub event_seq: u64,
    pub event_hash: u64,
    pub artifact_id: u64,
    pub artifact_receipt_hash: u64,
    pub artifact_path_hash: u64,
    pub artifact_content_hash: u64,
    pub artifact_bytes: u64,
    pub sandbox_root_hash: u64,
}

impl ToolEffectReceipt {
    pub fn from_persisted_event(
        record: &ToolExecutionRecord,
        event: &ControlEvent,
    ) -> Option<Self> {
        if !persisted_execution_effect_is_valid(record, event) {
            return None;
        }

        let effect_hash = tool_effect_output_hash(event.state_before.packet, event.state_after.packet);
        let effect = Effect::artifact(
            effect_hash,
            record.receipt.artifact_path_hash,
            record.receipt.artifact_content_hash,
            record.receipt.artifact_bytes,
            record.receipt.sandbox_root_hash,
        );
        Some(Self {
            capability: record.request.capability,
            registry_policy_hash: record.request.registry_policy_hash,
            request_hash: record.request.contract_hash(),
            receipt_hash: record.receipt.receipt_hash,
            effect_hash,
            effect,
            event_seq: event.seq,
            event_hash: event.self_hash,
            artifact_id: event.state_after.packet.artifact_id,
            artifact_receipt_hash: event.state_after.packet.artifact_receipt_hash,
            artifact_path_hash: record.receipt.artifact_path_hash,
            artifact_content_hash: record.receipt.artifact_content_hash,
            artifact_bytes: record.receipt.artifact_bytes,
            sandbox_root_hash: record.receipt.sandbox_root_hash,
        })
    }

    pub fn is_valid(self) -> bool {
        self.capability == CapabilityId::Tooling
            && self.registry_policy_hash != 0
            && self.request_hash != 0
            && self.receipt_hash != 0
            && self.effect_hash != 0
            && self.effect.is_valid()
            && self.effect.kind == ToolEffectKind::Artifact
            && self.effect.digest == self.effect_hash
            && self.effect
                == Effect::artifact(
                    self.effect_hash,
                    self.artifact_path_hash,
                    self.artifact_content_hash,
                    self.artifact_bytes,
                    self.sandbox_root_hash,
                )
            && self.event_seq != 0
            && self.event_hash != 0
            && self.artifact_id != 0
            && self.artifact_receipt_hash != 0
    }

    pub fn is_sandbox_artifact_bound(self) -> bool {
        self.effect.kind == ToolEffectKind::Artifact
            && self.artifact_path_hash != 0
            && self.artifact_content_hash != 0
            && self.artifact_bytes != 0
            && self.sandbox_root_hash != 0
    }

    pub fn replay_verified(self, tlog: &TLog) -> bool {
        self.is_valid() && tlog.iter().any(|event| self.matches_event(event))
    }

    pub fn replay_verified_with_registry(self, tlog: &TLog, registry: CapabilityRegistry) -> bool {
        self.registry_policy_hash == registry.policy_hash()
            && tlog.iter().any(|event| self.matches_event(event))
    }

    pub fn matches_event(self, event: &ControlEvent) -> bool {
        event.seq == self.event_seq
            && event.self_hash == self.event_hash
            && self.capability == CapabilityId::Tooling
            && self.registry_policy_hash != 0
            && event.capability_registry_projection.policy_hash == self.registry_policy_hash
            && event.state_after.packet.artifact_id == self.artifact_id
            && event.state_after.packet.artifact_receipt_hash == self.artifact_receipt_hash
            && self.effect.digest == self.effect_hash
            && tool_effect_output_hash(event.state_before.packet, event.state_after.packet)
                == self.effect_hash
    }
}

impl EvidenceProducer for ToolExecutionRecord {
    type Record = ToolExecutionRecord;

    fn record(&self) -> &Self::Record {
        self
    }

    fn submission(&self) -> EvidenceSubmission {
        ToolExecutionRecord::submission(self)
    }
}

pub fn append_tool_effect_receipt_ndjson(
    path: impl AsRef<Path>,
    receipt: &ToolEffectReceipt,
) -> Result<(), ToolSandboxError> {
    if !receipt.is_valid() {
        return Err(ToolSandboxError::InvalidToolReceipt);
    }

    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|_| ToolSandboxError::SandboxIo)?;
        }
    }

    {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|_| ToolSandboxError::SandboxIo)?;
        writeln!(file, "{}", encode_tool_effect_receipt_ndjson(*receipt))
            .map_err(|_| ToolSandboxError::SandboxIo)?;
        file.sync_all().map_err(|_| ToolSandboxError::SandboxIo)?;
    }

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            sync_dir(parent)?;
        }
    }

    Ok(())
}

pub fn load_tool_effect_receipts_ndjson(
    path: impl AsRef<Path>,
) -> Result<Vec<ToolEffectReceipt>, ToolSandboxError> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(path).map_err(|_| ToolSandboxError::SandboxIo)?;
    let reader = BufReader::new(file);
    let mut receipts = Vec::new();

    for line in reader.lines() {
        let line = line.map_err(|_| ToolSandboxError::SandboxIo)?;
        if line.trim().is_empty() {
            continue;
        }
        receipts.push(decode_tool_effect_receipt_ndjson(&line)?);
    }

    Ok(receipts)
}

pub fn verify_tool_effect_receipts(
    tlog: &TLog,
    receipts: &[ToolEffectReceipt],
) -> Result<usize, ToolSandboxError> {
    for receipt in receipts {
        if !receipt.replay_verified(tlog) {
            return Err(ToolSandboxError::InvalidReplay);
        }
    }

    Ok(receipts.len())
}

pub fn encode_tool_effect_receipt_ndjson(receipt: ToolEffectReceipt) -> String {
    let fields = [
        TOOL_EFFECT_RECEIPT_SCHEMA_VERSION,
        TOOL_EFFECT_RECEIPT_RECORD,
        receipt.capability as u64,
        receipt.registry_policy_hash,
        receipt.request_hash,
        receipt.receipt_hash,
        receipt.effect_hash,
        receipt.effect.kind as u64,
        receipt.effect.digest,
        receipt.effect.metadata,
        receipt.event_seq,
        receipt.event_hash,
        receipt.artifact_id,
        receipt.artifact_receipt_hash,
        receipt.artifact_path_hash,
        receipt.artifact_content_hash,
        receipt.artifact_bytes,
        receipt.sandbox_root_hash,
    ];
    let body = fields
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{body}]")
}

pub fn decode_tool_effect_receipt_ndjson(
    line: &str,
) -> Result<ToolEffectReceipt, ToolSandboxError> {
    let trimmed = line.trim();
    let body = trimmed
        .strip_prefix('[')
        .and_then(|v| v.strip_suffix(']'))
        .ok_or(ToolSandboxError::InvalidToolReceiptRecord)?;
    let fields = if body.trim().is_empty() {
        Vec::new()
    } else {
        body.split(',')
            .map(|raw| {
                raw.trim()
                    .parse::<u64>()
                    .map_err(|_| ToolSandboxError::InvalidToolReceiptRecord)
            })
            .collect::<Result<Vec<_>, _>>()?
    };

    if fields.len() != 18
        || fields[0] != TOOL_EFFECT_RECEIPT_SCHEMA_VERSION
        || fields[1] != TOOL_EFFECT_RECEIPT_RECORD
    {
        return Err(ToolSandboxError::InvalidToolReceiptRecord);
    }

    let effect = Effect {
        kind: tool_effect_kind_from_u64(fields[7])?,
        digest: fields[8],
        metadata: fields[9],
    };

    let receipt = ToolEffectReceipt {
        capability: capability_from_u64(fields[2])?,
        registry_policy_hash: fields[3],
        request_hash: fields[4],
        receipt_hash: fields[5],
        effect_hash: fields[6],
        effect,
        event_seq: fields[10],
        event_hash: fields[11],
        artifact_id: fields[12],
        artifact_receipt_hash: fields[13],
        artifact_path_hash: fields[14],
        artifact_content_hash: fields[15],
        artifact_bytes: fields[16],
        sandbox_root_hash: fields[17],
    };

    if !receipt.is_valid() {
        return Err(ToolSandboxError::InvalidToolReceipt);
    }

    Ok(receipt)
}

fn tooling_payload_hash(record: &ToolExecutionRecord) -> u64 {
    let mut h = 0xa54f_f53a_5f1d_36f1u64;
    h = mix(h, record.request.contract_hash());
    h = mix(h, record.request.registry_policy_hash);
    h = mix(h, record.receipt.registry_policy_hash);
    h = mix(h, record.receipt.objective_id);
    h = mix(h, record.receipt.task_id);
    h = mix(h, record.receipt.command_hash);
    h = mix(h, record.receipt.input_hash);
    h = mix(h, record.receipt.output_hash);
    h = mix(h, record.receipt.effect.kind as u64);
    h = mix(h, record.receipt.effect.digest);
    h = mix(h, record.receipt.effect.metadata);
    h = mix(h, record.receipt.exit_code as u64);
    h = mix(h, record.receipt.artifact_path_hash);
    h = mix(h, record.receipt.artifact_content_hash);
    h = mix(h, record.receipt.artifact_bytes);
    h = mix(h, record.receipt.sandbox_root_hash);
    h = mix(h, record.receipt.receipt_hash);
    h.max(1)
}

fn persisted_execution_effect_is_valid(record: &ToolExecutionRecord, event: &ControlEvent) -> bool {
    if !record.is_valid()
        || event.from != Phase::Execute
        || event.to != Phase::Execute
        || event.kind != EventKind::Persisted
        || event.cause != Cause::EvidenceSubmitted
        || event.evidence != Evidence::ArtifactReceipt
        || event.decision != Decision::Continue
        || event.failure.is_some()
        || event.recovery_action.is_some()
        || event.affected_gate != Some(GateId::Execution)
        || event.capability_registry_projection.policy_hash != record.request.registry_policy_hash
        || !record.request.matches_packet(event.state_before.packet)
    {
        return false;
    }

    let mut expected = event.state_before;
    expected.packet.materialize_artifact();
    expected.apply_evidence(GateId::Execution, Evidence::ArtifactReceipt, true);

    event.state_after == expected
        && event.state_after.gates.execution.status == GateStatus::Pass
        && event.state_after.gates.execution.evidence == Evidence::ArtifactReceipt
        && record.receipt.output_hash
            == tool_effect_output_hash(event.state_before.packet, event.state_after.packet)
}

fn expected_receipt_hash_for_effect(
    request: ToolRequest,
    exit_code: u8,
    effect: Effect,
) -> u64 {
    let mut h = 0x3c6ef372fe94f82bu64;
    h = mix(h, request.contract_hash());
    h = mix(h, effect.kind as u64);
    h = mix(h, effect.digest);
    h = mix(h, effect.metadata);
    h = mix(h, effect.contract_hash());
    h = mix(h, exit_code as u64);
    h.max(1)
}

fn tool_command_hash(packet: Packet) -> u64 {
    let mut h = 0xbb67ae8584caa73bu64;
    h = mix(h, packet.objective_id);
    h = mix(h, packet.active_task_id);
    h.max(1)
}

fn tool_input_hash(packet: Packet) -> u64 {
    let mut h = 0x13198a2e03707344u64;
    h = mix(h, packet.objective_id);
    h = mix(h, packet.active_task_id);
    h = mix(h, packet.ready_tasks as u64);
    h = mix(h, packet.revision);
    h.max(1)
}

fn tool_output_hash(request: ToolRequest) -> u64 {
    let mut h = 0x243f6a8885a308d3u64;
    h = mix(h, request.objective_id);
    h = mix(h, request.task_id);
    h = mix(h, request.command_hash);
    h = mix(h, request.input_hash);
    h.max(1)
}

fn tool_effect_output_hash(before: Packet, after: Packet) -> u64 {
    let mut h = 0x4528_21e6_38d0_1377u64;
    h = mix_packet(h, before);
    h = mix_packet(h, after);
    h.max(1)
}

fn mix_packet(mut h: u64, packet: Packet) -> u64 {
    h = mix(h, packet.objective_id);
    h = mix(h, packet.objective_required_tasks as u64);
    h = mix(h, packet.objective_done_tasks as u64);
    h = mix(h, packet.ready_tasks as u64);
    h = mix(h, packet.active_task_id);
    h = mix(h, packet.artifact_id);
    h = mix(h, packet.parent_artifact_id);
    h = mix(h, packet.artifact_bytes);
    h = mix(h, packet.artifact_receipt_hash);
    h = mix(h, packet.artifact_lineage_hash);
    h = mix(h, packet.revision);
    h
}

fn tool_failure_hash(request: ToolRequest, exit_code: u8) -> u64 {
    let mut h = 0x9e3779b97f4a7c15u64;
    h = mix(h, request.contract_hash());
    h = mix(h, exit_code as u64);
    h.max(1)
}

fn artifact_relative_name(request: ToolRequest) -> String {
    format!(
        "artifact-{:016x}-{:016x}.canon",
        request.contract_hash(),
        request.input_hash
    )
}

fn sandbox_artifact_body(request: ToolRequest, before: Packet, after: Packet) -> Vec<u8> {
    format!(
        concat!(
            "canon-tool-artifact-v1\n",
            "capability={}\n",
            "registry_policy_hash={}\n",
            "request_hash={}\n",
            "objective_id={}\n",
            "task_id={}\n",
            "input_hash={}\n",
            "effect=MaterializeArtifact\n",
            "before_revision={}\n",
            "after_revision={}\n",
            "artifact_id={}\n",
            "artifact_receipt_hash={}\n"
        ),
        request.capability as u64,
        request.registry_policy_hash,
        request.contract_hash(),
        request.objective_id,
        request.task_id,
        request.input_hash,
        before.revision,
        after.revision,
        after.artifact_id,
        after.artifact_receipt_hash
    )
    .into_bytes()
}

fn ensure_relative_name(name: &str) -> Result<(), ToolSandboxError> {
    let path = Path::new(name);
    if path.components().count() == 1 && !name.is_empty() && !name.contains(std::path::MAIN_SEPARATOR)
    {
        Ok(())
    } else {
        Err(ToolSandboxError::PathEscapesSandbox)
    }
}

fn ensure_under(root: &Path, path: &Path) -> Result<(), ToolSandboxError> {
    let parent = path.parent().ok_or(ToolSandboxError::SandboxIo)?;
    fs::create_dir_all(parent).map_err(|_| ToolSandboxError::SandboxIo)?;
    let parent = parent
        .canonicalize()
        .map_err(|_| ToolSandboxError::SandboxIo)?;
    if parent.starts_with(root) {
        Ok(())
    } else {
        Err(ToolSandboxError::PathEscapesSandbox)
    }
}

fn sync_dir(path: &Path) -> Result<(), ToolSandboxError> {
    let dir = File::open(path).map_err(|_| ToolSandboxError::SandboxIo)?;
    dir.sync_all().map_err(|_| ToolSandboxError::SandboxIo)
}

fn bytes_hash(bytes: &[u8]) -> u64 {
    let mut h = 0x6a09e667f3bcc909u64;
    h = mix(h, bytes.len() as u64);
    for byte in bytes {
        h = mix(h, *byte as u64);
    }
    h.max(1)
}

fn path_hash(path: impl AsRef<[u8]>) -> u64 {
    bytes_hash(path.as_ref())
}


fn string_hash(value: &str) -> u64 {
    bytes_hash(value.as_bytes())
}

fn argv_hash(command: &str, args: &[&str]) -> u64 {
    let mut h = 0x1f83_d9ab_fb41_bd6bu64;
    h = mix(h, string_hash(command));
    h = mix(h, args.len() as u64);
    for arg in args {
        h = mix(h, string_hash(arg));
    }
    h.max(1)
}

fn locked_env_hash(env: &[(String, String)]) -> u64 {
    let mut h = 0x5be0_cd19_137e_2179u64;
    h = mix(h, env.len() as u64);
    for (key, value) in env {
        h = mix(h, string_hash(key));
        h = mix(h, string_hash(value));
    }
    h.max(1)
}

fn expected_process_receipt_hash(receipt: &SandboxProcessReceipt) -> u64 {
    let mut h = 0xcbbb_9d5d_c105_9ed8u64;
    h = mix(h, receipt.request_hash);
    h = mix(h, receipt.registry_policy_hash);
    h = mix(h, receipt.command_hash);
    h = mix(h, receipt.argv_hash);
    h = mix(h, receipt.cwd_hash);
    h = mix(h, receipt.env_hash);
    h = mix(h, receipt.timeout_ms);
    h = mix(h, receipt.max_output_bytes);
    h = mix(h, receipt.effect.kind as u64);
    h = mix(h, receipt.effect.digest);
    h = mix(h, receipt.effect.metadata);
    h = mix(h, receipt.effect.contract_hash());
    h = mix(h, receipt.stdout_hash);
    h = mix(h, receipt.stderr_hash);
    h = mix(h, receipt.stdout_bytes);
    h = mix(h, receipt.stderr_bytes);
    h = mix(h, receipt.exit_status);
    h = mix(h, receipt.timed_out as u64);
    h.max(1)
}

fn ensure_process_token(value: &str) -> Result<(), ToolSandboxError> {
    if value.is_empty() || value.contains('\0') || value.contains('=') {
        Err(ToolSandboxError::InvalidCommand)
    } else {
        Ok(())
    }
}

fn ensure_relative_process_path(path: &str) -> Result<(), ToolSandboxError> {
    let path = Path::new(path);
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(ToolSandboxError::PathEscapesSandbox);
    }
    Ok(())
}

fn bounded_file_bytes(path: &Path, max_bytes: u64) -> Result<Vec<u8>, ToolSandboxError> {
    let metadata = fs::metadata(path).map_err(|_| ToolSandboxError::SandboxIo)?;
    if metadata.len() > max_bytes {
        return Err(ToolSandboxError::OutputTooLarge);
    }

    let mut file = File::open(path).map_err(|_| ToolSandboxError::SandboxIo)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|_| ToolSandboxError::SandboxIo)?;
    if bytes.len() as u64 > max_bytes {
        return Err(ToolSandboxError::OutputTooLarge);
    }
    Ok(bytes)
}

fn capability_from_u64(value: u64) -> Result<CapabilityId, ToolSandboxError> {
    match value {
        7 => Ok(CapabilityId::Tooling),
        _ => Err(ToolSandboxError::InvalidToolReceiptRecord),
    }
}

fn tool_effect_kind_from_u64(value: u64) -> Result<ToolEffectKind, ToolSandboxError> {
    match value {
        0 => Ok(ToolEffectKind::None),
        1 => Ok(ToolEffectKind::Artifact),
        2 => Ok(ToolEffectKind::Process),
        _ => Err(ToolSandboxError::InvalidToolReceiptRecord),
    }
}