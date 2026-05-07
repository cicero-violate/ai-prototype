//! Shared deterministic transport primitives for OpenAI-compatible LLM providers.
//!
//! Provider modules keep their provider-specific request/response schemas and
//! receipt types. This module owns the duplicated local endpoint and identity
//! hashing rules so provider clients cannot drift silently.

use crate::kernel::mix;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalLlmEndpoint {
    pub host: String,
    pub port: u16,
    pub path_prefix: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocalEndpointError {
    InvalidUrl,
    NonLocalHost,
}

pub fn parse_local_http_endpoint(base_url: &str) -> Result<LocalLlmEndpoint, LocalEndpointError> {
    let without_scheme = base_url
        .trim()
        .strip_prefix("http://")
        .ok_or(LocalEndpointError::InvalidUrl)?;
    let (host_port, path) = without_scheme
        .split_once('/')
        .map_or((without_scheme, ""), |(host_port, path)| (host_port, path));
    let (host, port) = host_port
        .rsplit_once(':')
        .ok_or(LocalEndpointError::InvalidUrl)?;
    if host != "127.0.0.1" && host != "localhost" {
        return Err(LocalEndpointError::NonLocalHost);
    }
    let port = port
        .parse::<u16>()
        .map_err(|_| LocalEndpointError::InvalidUrl)?;
    if port == 0 {
        return Err(LocalEndpointError::InvalidUrl);
    }
    let path_prefix = if path.trim().is_empty() {
        String::new()
    } else {
        format!("/{}", path.trim_matches('/'))
    };
    Ok(LocalLlmEndpoint {
        host: host.to_string(),
        port,
        path_prefix,
    })
}

pub fn chat_completions_path(path_prefix: &str) -> String {
    let prefix = path_prefix.trim_end_matches('/');
    if prefix.is_empty() {
        "/v1/chat/completions".to_string()
    } else {
        format!("{prefix}/chat/completions")
    }
}

pub fn provider_text_hash(text: &str) -> u64 {
    let mut h = 0x6c6c_6d5f_7472_616eu64;
    for b in text.as_bytes() {
        h = mix(h, *b as u64);
    }
    h.max(1)
}

pub fn provider_config_hash(provider: &str, base_url: &str, model: &str) -> u64 {
    let mut h = 0x6c6c_6d5f_636f_6e66u64;
    h = mix(h, provider_text_hash(provider));
    h = mix(h, provider_text_hash(base_url));
    h = mix(h, provider_text_hash(model));
    h.max(1)
}

pub fn retry_policy_hash(seed: u64, timeout_ms: u64, max_retries: u32, attempt_budget: u32) -> u64 {
    let mut h = seed;
    h = mix(h, timeout_ms);
    h = mix(h, max_retries as u64);
    h = mix(h, attempt_budget as u64);
    h.max(1)
}

pub fn request_identity_hash(
    seed: u64,
    provider_hash: u64,
    base_url_hash: u64,
    model_id: u64,
    request_hash: u64,
) -> u64 {
    let mut h = seed;
    h = mix(h, provider_hash);
    h = mix(h, base_url_hash);
    h = mix(h, model_id);
    h = mix(h, request_hash);
    h.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_parser_accepts_local_openai_compatible_base_urls() {
        let endpoint = parse_local_http_endpoint("http://127.0.0.1:11434/v1").unwrap();
        assert_eq!(endpoint.host, "127.0.0.1");
        assert_eq!(endpoint.port, 11434);
        assert_eq!(endpoint.path_prefix, "/v1");
        assert_eq!(
            chat_completions_path(&endpoint.path_prefix),
            "/v1/chat/completions"
        );
    }

    #[test]
    fn endpoint_parser_rejects_non_local_hosts_and_zero_ports() {
        assert_eq!(
            parse_local_http_endpoint("http://example.com:11434/v1"),
            Err(LocalEndpointError::NonLocalHost)
        );
        assert_eq!(
            parse_local_http_endpoint("http://127.0.0.1:0/v1"),
            Err(LocalEndpointError::InvalidUrl)
        );
    }

    #[test]
    fn shared_hashes_are_deterministic_and_provider_scoped() {
        let ollama = provider_config_hash("ollama", "http://127.0.0.1:11434/v1", "qwen");
        let openai = provider_config_hash("openai-compatible", "http://127.0.0.1:11434/v1", "qwen");
        assert_ne!(ollama, openai);
        assert_eq!(
            ollama,
            provider_config_hash("ollama", "http://127.0.0.1:11434/v1", "qwen")
        );
    }
}
