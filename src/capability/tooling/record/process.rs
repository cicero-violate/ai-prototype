use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::capability::{
    CapabilityId, CapabilityRegistry, EvidenceProducer, EvidenceSubmission, PacketEffect,
};
use crate::kernel::{mix, Evidence, GateId};

use super::hash::{
    argv_hash, bounded_file_bytes, bytes_hash, ensure_process_token, ensure_relative_process_path,
    ensure_under, locked_env_hash, parse_u64_ndjson_fields, path_hash, string_hash, sync_dir,
    tool_effect_kind_from_u64, validate_u64_ndjson_header,
};
use super::request::SandboxProcessRequest;
use super::types::{
    Effect, ToolSandboxError, SANDBOX_PROCESS_RECEIPT_RECORD,
    SANDBOX_PROCESS_RECEIPT_SCHEMA_VERSION,
};

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
    pub fn contract_hash(&self) -> u64 {
        self.receipt_hash
    }

    pub fn is_success(&self) -> bool {
        self.is_contract_valid() && self.exit_status == 0 && !self.timed_out
    }

    pub fn is_contract_valid(&self) -> bool {
        self.request_hash != 0
            && self.registry_policy_hash != 0
            && self.command_hash != 0
            && self.argv_hash != 0
            && self.cwd_hash != 0
            && self.env_hash != 0
            && self.timeout_ms != 0
            && self.max_output_bytes != 0
            && self.effect.is_valid()
            && self.effect_is_normalized()
            && self.stdout_hash != 0
            && self.stderr_hash != 0
            && self.stdout_bytes <= self.max_output_bytes
            && self.stderr_bytes <= self.max_output_bytes
            && self.receipt_hash == expected_process_receipt_hash(self)
    }

    pub fn submission(&self) -> EvidenceSubmission {
        EvidenceSubmission::with_effect_payload(
            GateId::Execution,
            Evidence::ExecutionReceipt,
            self.is_success(),
            PacketEffect::None,
            process_payload_hash(self),
        )
    }

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
                Evidence::ExecutionReceipt,
                PacketEffect::None,
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
    let fields = parse_u64_ndjson_fields(line)?;
    validate_u64_ndjson_header(
        &fields,
        20,
        SANDBOX_PROCESS_RECEIPT_SCHEMA_VERSION,
        SANDBOX_PROCESS_RECEIPT_RECORD,
    )?;

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

fn process_payload_hash(receipt: &SandboxProcessReceipt) -> u64 {
    let mut h = 0x2157_2e63_a9b4_c1d5u64;
    h = crate::kernel::mix(h, receipt.request_hash);
    h = crate::kernel::mix(h, receipt.registry_policy_hash);
    h = crate::kernel::mix(h, receipt.command_hash);
    h = crate::kernel::mix(h, receipt.argv_hash);
    h = crate::kernel::mix(h, receipt.cwd_hash);
    h = crate::kernel::mix(h, receipt.env_hash);
    h = crate::kernel::mix(h, receipt.timeout_ms);
    h = crate::kernel::mix(h, receipt.max_output_bytes);
    h = crate::kernel::mix(h, receipt.effect.kind as u64);
    h = crate::kernel::mix(h, receipt.effect.digest);
    h = crate::kernel::mix(h, receipt.effect.metadata);
    h = crate::kernel::mix(h, receipt.stdout_hash);
    h = crate::kernel::mix(h, receipt.stderr_hash);
    h = crate::kernel::mix(h, receipt.stdout_bytes);
    h = crate::kernel::mix(h, receipt.stderr_bytes);
    h = crate::kernel::mix(h, receipt.exit_status);
    h = crate::kernel::mix(h, receipt.timed_out as u64);
    h = crate::kernel::mix(h, receipt.receipt_hash);
    h.max(1)
}

impl EvidenceProducer for SandboxProcessReceipt {
    type Record = SandboxProcessReceipt;

    fn record(&self) -> &Self::Record {
        self
    }

    fn submission(&self) -> EvidenceSubmission {
        SandboxProcessReceipt::submission(self)
    }
}