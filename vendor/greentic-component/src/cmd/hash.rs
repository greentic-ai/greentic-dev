use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Args, Parser};
use serde_json::Value;

use crate::path_safety::normalize_under_root;

#[derive(Args, Debug, Clone)]
#[command(about = "Recompute the wasm hash inside component.manifest.json")]
pub struct HashArgs {
    /// Path to component.manifest.json
    #[arg(default_value = "component.manifest.json")]
    pub manifest: PathBuf,
    /// Optional override for the wasm artifact path
    #[arg(long)]
    pub wasm: Option<PathBuf>,
}

#[derive(Parser, Debug)]
struct HashCli {
    #[command(flatten)]
    args: HashArgs,
}

pub fn parse_from_cli() -> HashArgs {
    HashCli::parse().args
}

pub fn run(args: HashArgs) -> Result<()> {
    let workspace_root = std::env::current_dir()
        .context("failed to read current directory")?
        .canonicalize()
        .context("failed to canonicalize workspace root")?;
    let manifest_path =
        normalize_or_canonicalize(&workspace_root, &args.manifest).with_context(|| {
            format!(
                "manifest path escapes workspace root: {}",
                args.manifest.display()
            )
        })?;
    let manifest_text = fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let mut manifest: Value = serde_json::from_str(&manifest_text)
        .with_context(|| format!("invalid json: {}", manifest_path.display()))?;
    let manifest_root = manifest_path
        .parent()
        .unwrap_or(workspace_root.as_path())
        .canonicalize()
        .with_context(|| {
            format!(
                "failed to canonicalize manifest directory {}",
                manifest_path.display()
            )
        })?;
    let wasm_candidate = resolve_wasm_path(&manifest, args.wasm.as_deref())?;
    let wasm_path =
        normalize_or_canonicalize(&manifest_root, &wasm_candidate).with_context(|| {
            format!(
                "wasm path escapes manifest root {}",
                manifest_root.display()
            )
        })?;
    let wasm_bytes = fs::read(&wasm_path)
        .with_context(|| format!("failed to read wasm at {}", wasm_path.display()))?;
    let digest = blake3::hash(&wasm_bytes).to_hex().to_string();
    manifest["hashes"]["component_wasm"] = Value::String(format!("blake3:{digest}"));
    let formatted = serde_json::to_string_pretty(&manifest)?;
    fs::write(&manifest_path, formatted + "\n")
        .with_context(|| format!("failed to write {}", manifest_path.display()))?;
    println!(
        "Updated {} with hash of {}",
        manifest_path.display(),
        wasm_path.display()
    );
    Ok(())
}

fn resolve_wasm_path(manifest: &Value, override_path: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = override_path {
        return Ok(path.to_path_buf());
    }
    let artifact = manifest
        .get("artifacts")
        .and_then(|art| art.get("component_wasm"))
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("manifest is missing artifacts.component_wasm"))?;
    Ok(PathBuf::from(artifact))
}

fn normalize_or_canonicalize(root: &Path, candidate: &Path) -> Result<PathBuf> {
    if candidate.is_absolute() {
        return candidate
            .canonicalize()
            .with_context(|| format!("failed to canonicalize {}", candidate.display()));
    }
    normalize_under_root(root, candidate)
}
