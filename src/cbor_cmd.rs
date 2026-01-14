use std::fs;

use anyhow::{Context, Result};

use crate::cli::CborArgs;

pub fn run(args: CborArgs) -> Result<()> {
    let data = fs::read(&args.path)
        .with_context(|| format!("failed to read CBOR file {}", args.path.display()))?;
    let value: serde_cbor::Value =
        serde_cbor::from_slice(&data).context("failed to decode CBOR payload")?;
    let rendered = serde_json::to_string_pretty(&value).context("failed to render CBOR as JSON")?;
    println!("{rendered}");
    Ok(())
}
