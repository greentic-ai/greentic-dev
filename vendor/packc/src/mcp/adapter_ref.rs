#[derive(Debug, Clone)]
pub struct McpAdapterRef {
    pub protocol: &'static str,
    pub image: &'static str,
    pub digest: Option<&'static str>,
}

/// Pinned MCP adapter reference for protocol 25.06.18.
/// Image tag is fixed; digest is provided for verification when pulled remotely.
pub const MCP_ADAPTER_25_06_18: McpAdapterRef = McpAdapterRef {
    protocol: "25.06.18",
    image: "ghcr.io/greentic-ai/greentic-mcp-adapter:25.06.18-v0.4.4",
    digest: Some("sha256:2090ee1905413eb9fbc6bf6c8bf0317f13c17890e698382bd5e8675ed241417d"),
};
