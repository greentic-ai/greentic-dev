use std::path::{Path, PathBuf};

use anyhow::Result;

use super::adapter_ref::{MCP_ADAPTER_25_06_18, McpAdapterRef};

/// Return the local adapter path for the given reference.
///
/// Current behaviour: use the vendored asset bundled in packc.
/// Future: implement OCI pull + cache when GHCR is the source of truth.
pub fn ensure_adapter_local(adapter: &McpAdapterRef) -> Result<PathBuf> {
    if adapter.protocol == MCP_ADAPTER_25_06_18.protocol {
        vendored_adapter_path()
    } else {
        anyhow::bail!("unsupported MCP adapter protocol `{}`", adapter.protocol)
    }
}

fn vendored_adapter_path() -> Result<PathBuf> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("mcp_adapter_25_06_18.component.wasm");
    if path.exists() {
        Ok(path)
    } else {
        Err(anyhow::anyhow!(
            "vendored MCP adapter missing at {}",
            path.display()
        ))
    }
}
