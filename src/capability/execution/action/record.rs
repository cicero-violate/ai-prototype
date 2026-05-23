use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::time::Duration;

use crate::capability::{
    CapabilityId, CapabilityRegistry, EvidenceProducer, EvidenceSubmission, PacketEffect,
};
use crate::kernel::{mix, Evidence, GateId};

use crate::capability::execution::record::hash::{
    bytes_hash, parse_u64_ndjson_fields, string_hash, sync_dir, tool_effect_kind_from_u64,
    validate_u64_ndjson_header,
};
use crate::capability::execution::record::types::{Effect, ToolSandboxError};

pub const ACTION_RECEIPT_SCHEMA_VERSION: u64 = 1;
pub const ACTION_RECEIPT_RECORD: u64 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActionCallRequest {
    pub capability: CapabilityId,
    pub registry_policy_hash: u64,
    pub worker_url_hash: u64,
    pub tool_name_hash: u64,
    pub args_hash: u64,
    pub timeout_ms: u64,
    pub max_output_bytes: u64,
}

impl ActionCallRequest {
    pub fn new(
        registry: CapabilityRegistry,
        worker_url: &str,
        tool_name: &str,
        args_json: &str,
        timeout_ms: u64,
        max_output_bytes: u64,
    ) -> Self {
        Self {
            capability: CapabilityId::Tooling,
            registry_policy_hash: registry.policy_hash(),
            worker_url_hash: string_hash(worker_url),
            tool_name_hash: string_hash(tool_name),
            args_hash: string_hash(args_json),
            timeout_ms,
            max_output_bytes,
        }
    }

    pub fn contract_hash(&self) -> u64 {
        let mut h = 0x8c9d_f84d_7b6a_12e3u64;
        h = mix(h, self.capability as u64);
        h = mix(h, self.registry_policy_hash);
        h = mix(h, self.worker_url_hash);
        h = mix(h, self.tool_name_hash);
        h = mix(h, self.args_hash);
        h = mix(h, self.timeout_ms);
        h = mix(h, self.max_output_bytes);
        h.max(1)
    }

    pub fn is_admissible(&self) -> bool {
        self.capability == CapabilityId::Tooling
            && self.registry_policy_hash != 0
            && self.worker_url_hash != 0
            && self.tool_name_hash != 0
            && self.args_hash != 0
            && self.timeout_ms != 0
            && self.max_output_bytes != 0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionReceipt {
    pub request_hash: u64,
    pub registry_policy_hash: u64,
    pub worker_url_hash: u64,
    pub tool_name_hash: u64,
    pub args_hash: u64,
    pub timeout_ms: u64,
    pub max_output_bytes: u64,
    pub effect: Effect,
    pub response_hash: u64,
    pub response_bytes: u64,
    pub exit_status: u64,
    pub timed_out: bool,
    pub receipt_hash: u64,
}

impl ActionReceipt {
    pub fn from_response(
        request: &ActionCallRequest,
        response: &[u8],
        exit_status: u64,
        timed_out: bool,
    ) -> Self {
        let response_hash = bytes_hash(response);
        let response_bytes = response.len() as u64;
        let effect = Effect::process(response_hash, 0, response_bytes, 0, exit_status, timed_out);
        let mut receipt = Self {
            request_hash: request.contract_hash(),
            registry_policy_hash: request.registry_policy_hash,
            worker_url_hash: request.worker_url_hash,
            tool_name_hash: request.tool_name_hash,
            args_hash: request.args_hash,
            timeout_ms: request.timeout_ms,
            max_output_bytes: request.max_output_bytes,
            effect,
            response_hash,
            response_bytes,
            exit_status,
            timed_out,
            receipt_hash: 0,
        };
        receipt.receipt_hash = expected_action_receipt_hash(&receipt);
        receipt
    }

    pub fn contract_hash(&self) -> u64 {
        self.receipt_hash
    }

    pub fn is_success(&self) -> bool {
        self.is_contract_valid() && self.exit_status == 0 && !self.timed_out
    }

    pub fn is_contract_valid(&self) -> bool {
        self.request_hash != 0
            && self.registry_policy_hash != 0
            && self.worker_url_hash != 0
            && self.tool_name_hash != 0
            && self.args_hash != 0
            && self.timeout_ms != 0
            && self.max_output_bytes != 0
            && self.effect.is_valid()
            && self.effect_is_normalized()
            && self.response_hash != 0
            && self.response_bytes <= self.max_output_bytes
            && self.receipt_hash == expected_action_receipt_hash(self)
    }

    pub fn effect_is_normalized(&self) -> bool {
        self.effect
            == Effect::process(
                self.response_hash,
                0,
                self.response_bytes,
                0,
                self.exit_status,
                self.timed_out,
            )
    }

    pub fn is_valid_for(&self, request: &ActionCallRequest) -> bool {
        request.is_admissible()
            && self.request_hash == request.contract_hash()
            && self.registry_policy_hash == request.registry_policy_hash
            && self.worker_url_hash == request.worker_url_hash
            && self.tool_name_hash == request.tool_name_hash
            && self.args_hash == request.args_hash
            && self.timeout_ms == request.timeout_ms
            && self.max_output_bytes == request.max_output_bytes
            && self.is_contract_valid()
    }

    pub fn submission(&self) -> EvidenceSubmission {
        EvidenceSubmission::with_effect_payload(
            GateId::Execution,
            Evidence::ExecutionReceipt,
            self.is_success(),
            PacketEffect::None,
            action_payload_hash(self),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveActionExecutor {
    pub worker_url: String,
    pub allowed_tools: Vec<String>,
    pub timeout_ms: u64,
    pub max_output_bytes: u64,
    pub registry: CapabilityRegistry,
}

impl LiveActionExecutor {
    pub fn new(worker_url: impl Into<String>) -> Self {
        Self {
            worker_url: worker_url.into(),
            allowed_tools: Vec::new(),
            timeout_ms: 1000,
            max_output_bytes: 64 * 1024,
            registry: CapabilityRegistry::canonical(),
        }
    }

    pub fn with_allowed_tool(mut self, tool_name: impl Into<String>) -> Self {
        self.allowed_tools.push(tool_name.into());
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

    pub fn request_for(
        &self,
        tool_name: &str,
        args_json: &str,
    ) -> Result<ActionCallRequest, ToolSandboxError> {
        self.validate_tool_name(tool_name)?;
        self.validate_args_json(args_json)?;
        Ok(ActionCallRequest::new(
            self.registry,
            &self.worker_url,
            tool_name,
            args_json,
            self.timeout_ms,
            self.max_output_bytes,
        ))
    }

    pub fn replay_receipt(
        &self,
        receipt: &ActionReceipt,
        tool_name: &str,
        args_json: &str,
    ) -> Result<bool, ToolSandboxError> {
        let request = self.request_for(tool_name, args_json)?;
        Ok(receipt.is_valid_for(&request))
    }

    pub fn execute_call(
        &self,
        tool_name: &str,
        args_json: &str,
    ) -> Result<ActionReceipt, ToolSandboxError> {
        let request = self.request_for(tool_name, args_json)?;
        let args_value: serde_json::Value =
            serde_json::from_str(args_json).map_err(|_| ToolSandboxError::InvalidCommand)?;
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": tool_name,
                "arguments": args_value,
            }
        });

        let response = reqwest::blocking::Client::builder()
            .timeout(Duration::from_millis(self.timeout_ms))
            .build()
            .map_err(|_| ToolSandboxError::SandboxIo)?
            .post(&self.worker_url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send();

        let (bytes, exit_status, timed_out) = match response {
            Ok(response) => {
                let status = response.status();
                let bytes = response
                    .bytes()
                    .map_err(|_| ToolSandboxError::SandboxIo)?
                    .to_vec();
                let exit_status = if status.is_success() { 0 } else { 1 };
                (bytes, exit_status, false)
            }
            Err(err) => {
                let timed_out = err.is_timeout();
                (Vec::new(), 1, timed_out)
            }
        };

        if bytes.len() as u64 > self.max_output_bytes {
            return Err(ToolSandboxError::OutputTooLarge);
        }

        Ok(ActionReceipt::from_response(
            &request,
            &bytes,
            exit_status,
            timed_out,
        ))
    }

    fn validate_tool_name(&self, tool_name: &str) -> Result<(), ToolSandboxError> {
        if tool_name.is_empty() || tool_name.contains('\0') {
            return Err(ToolSandboxError::InvalidCommand);
        }
        if self
            .allowed_tools
            .iter()
            .any(|allowed| allowed == tool_name)
        {
            Ok(())
        } else {
            Err(ToolSandboxError::CommandDenied)
        }
    }

    fn validate_args_json(&self, args_json: &str) -> Result<(), ToolSandboxError> {
        if args_json.len() as u64 > self.max_output_bytes {
            Err(ToolSandboxError::ArtifactTooLarge)
        } else {
            Ok(())
        }
    }
}

pub fn append_action_receipt_ndjson(
    path: impl AsRef<Path>,
    receipt: &ActionReceipt,
) -> Result<(), ToolSandboxError> {
    if !receipt.is_contract_valid() {
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
        writeln!(file, "{}", encode_action_receipt_ndjson(receipt))
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

pub fn load_action_receipts_ndjson(
    path: impl AsRef<Path>,
) -> Result<Vec<ActionReceipt>, ToolSandboxError> {
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
            receipts.push(decode_action_receipt_ndjson(&line)?);
        }
    }
    Ok(receipts)
}

pub fn verify_action_receipts(receipts: &[ActionReceipt]) -> Result<usize, ToolSandboxError> {
    for receipt in receipts {
        if !receipt.is_contract_valid() {
            return Err(ToolSandboxError::InvalidReplay);
        }
    }
    Ok(receipts.len())
}

pub fn encode_action_receipt_ndjson(receipt: &ActionReceipt) -> String {
    let fields = [
        ACTION_RECEIPT_SCHEMA_VERSION,
        ACTION_RECEIPT_RECORD,
        receipt.request_hash,
        receipt.registry_policy_hash,
        receipt.worker_url_hash,
        receipt.tool_name_hash,
        receipt.args_hash,
        receipt.timeout_ms,
        receipt.max_output_bytes,
        receipt.effect.kind as u64,
        receipt.effect.digest,
        receipt.effect.metadata,
        receipt.response_hash,
        receipt.response_bytes,
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

pub fn decode_action_receipt_ndjson(line: &str) -> Result<ActionReceipt, ToolSandboxError> {
    let fields = parse_u64_ndjson_fields(line)?;
    validate_u64_ndjson_header(
        &fields,
        17,
        ACTION_RECEIPT_SCHEMA_VERSION,
        ACTION_RECEIPT_RECORD,
    )?;

    let effect = Effect {
        kind: tool_effect_kind_from_u64(fields[9])?,
        digest: fields[10],
        metadata: fields[11],
    };

    let receipt = ActionReceipt {
        request_hash: fields[2],
        registry_policy_hash: fields[3],
        worker_url_hash: fields[4],
        tool_name_hash: fields[5],
        args_hash: fields[6],
        timeout_ms: fields[7],
        max_output_bytes: fields[8],
        effect,
        response_hash: fields[12],
        response_bytes: fields[13],
        exit_status: fields[14],
        timed_out: fields[15] != 0,
        receipt_hash: fields[16],
    };

    if !receipt.is_contract_valid() {
        return Err(ToolSandboxError::InvalidToolReceipt);
    }

    Ok(receipt)
}

fn expected_action_receipt_hash(receipt: &ActionReceipt) -> u64 {
    let mut h = 0x9142_6c18_4ed9_f3a5u64;
    h = mix(h, receipt.request_hash);
    h = mix(h, receipt.registry_policy_hash);
    h = mix(h, receipt.worker_url_hash);
    h = mix(h, receipt.tool_name_hash);
    h = mix(h, receipt.args_hash);
    h = mix(h, receipt.timeout_ms);
    h = mix(h, receipt.max_output_bytes);
    h = mix(h, receipt.effect.kind as u64);
    h = mix(h, receipt.effect.digest);
    h = mix(h, receipt.effect.metadata);
    h = mix(h, receipt.effect.contract_hash());
    h = mix(h, receipt.response_hash);
    h = mix(h, receipt.response_bytes);
    h = mix(h, receipt.exit_status);
    h = mix(h, receipt.timed_out as u64);
    h.max(1)
}

fn action_payload_hash(receipt: &ActionReceipt) -> u64 {
    let mut h = 0x7651_aa62_43ec_d91bu64;
    h = mix(h, receipt.request_hash);
    h = mix(h, receipt.registry_policy_hash);
    h = mix(h, receipt.worker_url_hash);
    h = mix(h, receipt.tool_name_hash);
    h = mix(h, receipt.args_hash);
    h = mix(h, receipt.timeout_ms);
    h = mix(h, receipt.max_output_bytes);
    h = mix(h, receipt.effect.kind as u64);
    h = mix(h, receipt.effect.digest);
    h = mix(h, receipt.effect.metadata);
    h = mix(h, receipt.response_hash);
    h = mix(h, receipt.response_bytes);
    h = mix(h, receipt.exit_status);
    h = mix(h, receipt.timed_out as u64);
    h = mix(h, receipt.receipt_hash);
    h.max(1)
}

impl EvidenceProducer for ActionReceipt {
    type Record = ActionReceipt;

    fn record(&self) -> &Self::Record {
        self
    }

    fn submission(&self) -> EvidenceSubmission {
        ActionReceipt::submission(self)
    }
}
