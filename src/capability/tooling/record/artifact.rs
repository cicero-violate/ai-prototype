use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::capability::{
    CapabilityRegistry, EvidenceSubmission, PacketEffect,
};
use crate::kernel::{
    mix, Cause, ControlEvent, Decision, EventKind, Evidence, GateId, GateStatus, Packet, Phase,
};

use super::hash::{
    artifact_relative_name, bytes_hash, ensure_relative_name, ensure_under, path_hash,
    sandbox_artifact_body, sync_dir, tool_effect_output_hash, tool_failure_hash, tool_output_hash,
};
use super::receipt::ToolEffectReceipt;
use super::request::ToolRequest;
use super::types::{Effect, ToolDecision, ToolEffectKind, ToolKind, ToolSandboxError};

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

pub(crate) fn persisted_execution_effect_is_valid(record: &ToolExecutionRecord, event: &ControlEvent) -> bool {
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