use std::env;

pub const DEFAULT_SUPERVISOR_PORT: u16 = 9100;

pub fn supervisor_port_from_env() -> Result<u16, String> {
    env::var("SUPERVISOR_PORT")
        .unwrap_or_else(|_| DEFAULT_SUPERVISOR_PORT.to_string())
        .parse::<u16>()
        .map_err(|_| "SUPERVISOR_PORT must be a u16".to_string())
}

pub fn default_mcp_connector_url(supervisor_port: u16) -> String {
    format!("http://127.0.0.1:{supervisor_port}")
}

pub fn mcp_connector_url_from_env(supervisor_port: u16) -> String {
    env::var("MCP_CONNECTOR_URL").unwrap_or_else(|_| default_mcp_connector_url(supervisor_port))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_mcp_connector_url_tracks_supervisor_port() {
        assert_eq!(default_mcp_connector_url(9100), "http://127.0.0.1:9100");
        assert_eq!(default_mcp_connector_url(9177), "http://127.0.0.1:9177");
    }
}
