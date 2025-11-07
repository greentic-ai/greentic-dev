use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use greentic_flow::flow_bundle::load_and_validate_bundle;

pub fn validate(path: &Path, compact_json: bool) -> Result<()> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("failed to read flow definition at {}", path.display()))?;

    let bundle = load_and_validate_bundle(&source, Some(path)).with_context(|| {
        format!(
            "flow validation failed for {} using greentic-flow",
            path.display()
        )
    })?;

    let serialized = if compact_json {
        serde_json::to_string(&bundle)?
    } else {
        serde_json::to_string_pretty(&bundle)?
    };

    println!("{serialized}");
    Ok(())
}
