use anyhow::{Context, Result};
use serde_json::Value;

pub fn parse_json_bytes(bytes: &[u8]) -> Result<Value> {
    let raw = std::str::from_utf8(bytes).context("component tool emitted non-UTF-8 JSON")?;
    let trimmed = raw.trim();
    anyhow::ensure!(
        !trimmed.is_empty(),
        "component tool did not emit JSON on stdout"
    );
    serde_json::from_str(trimmed).context("failed to parse JSON payload from component tool")
}
