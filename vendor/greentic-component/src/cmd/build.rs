#![cfg(feature = "cli")]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow, bail};
use clap::Args;
use serde_json::Value as JsonValue;

use crate::cmd::flow::{FlowScaffoldResult, scaffold_with_manifest};
use crate::config::{
    ConfigInferenceOptions, ConfigSchemaSource, load_manifest_with_schema, resolve_manifest_path,
};
use crate::path_safety::normalize_under_root;

const DEFAULT_MANIFEST: &str = "component.manifest.json";

#[derive(Args, Debug, Clone)]
pub struct BuildArgs {
    /// Path to component.manifest.json (or directory containing it)
    #[arg(long = "manifest", value_name = "PATH", default_value = DEFAULT_MANIFEST)]
    pub manifest: PathBuf,
    /// Path to the cargo binary (fallback: $CARGO, then `cargo` on PATH)
    #[arg(long = "cargo", value_name = "PATH")]
    pub cargo_bin: Option<PathBuf>,
    /// Overwrite existing flows without prompting
    #[arg(long = "force")]
    pub force: bool,
    /// Skip flow scaffolding
    #[arg(long = "no-flow")]
    pub no_flow: bool,
    /// Skip config inference; fail if config_schema is missing
    #[arg(long = "no-infer-config")]
    pub no_infer_config: bool,
    /// Do not write inferred config_schema back to the manifest
    #[arg(long = "no-write-schema")]
    pub no_write_schema: bool,
    /// Overwrite existing config_schema with inferred schema
    #[arg(long = "force-write-schema")]
    pub force_write_schema: bool,
    /// Skip schema validation
    #[arg(long = "no-validate")]
    pub no_validate: bool,
    /// Emit machine-readable JSON summary
    #[arg(long = "json")]
    pub json: bool,
}

#[derive(Debug, serde::Serialize)]
struct BuildSummary {
    manifest: PathBuf,
    wasm_path: PathBuf,
    wasm_hash: String,
    config_source: ConfigSchemaSource,
    schema_written: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    flows: Option<FlowScaffoldResult>,
}

pub fn run(args: BuildArgs) -> Result<()> {
    let manifest_path = resolve_manifest_path(&args.manifest);
    let cwd = std::env::current_dir().context("failed to read current directory")?;
    let manifest_path = if manifest_path.is_absolute() {
        manifest_path
    } else {
        cwd.join(manifest_path)
    };
    if !manifest_path.exists() {
        bail!("manifest not found at {}", manifest_path.display());
    }
    let cargo_bin = args
        .cargo_bin
        .clone()
        .or_else(|| std::env::var_os("CARGO").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("cargo"));
    let inference_opts = ConfigInferenceOptions {
        allow_infer: !args.no_infer_config,
        write_schema: !args.no_write_schema,
        force_write_schema: args.force_write_schema,
        validate: !args.no_validate,
    };
    println!(
        "Using manifest at {} (cargo: {})",
        manifest_path.display(),
        cargo_bin.display()
    );

    let config = load_manifest_with_schema(&manifest_path, &inference_opts)?;
    let flow_result = if args.no_flow {
        None
    } else {
        Some(scaffold_with_manifest(&config, args.force)?)
    };

    let manifest_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    build_wasm(manifest_dir, &cargo_bin)?;

    let mut manifest_to_write = config.manifest.clone();
    if !config.persist_schema {
        manifest_to_write
            .as_object_mut()
            .map(|obj| obj.remove("config_schema"));
    }
    let (wasm_path, wasm_hash) = update_manifest_hashes(manifest_dir, &mut manifest_to_write)?;
    write_manifest(&manifest_path, &manifest_to_write)?;

    if args.json {
        let payload = BuildSummary {
            manifest: manifest_path.clone(),
            wasm_path,
            wasm_hash,
            config_source: config.source,
            schema_written: config.schema_written && config.persist_schema,
            flows: flow_result,
        };
        serde_json::to_writer_pretty(std::io::stdout(), &payload)?;
        println!();
    } else {
        println!("Built wasm artifact at {}", wasm_path.display());
        println!("Updated {} hashes (blake3)", manifest_path.display());
        if config.schema_written && config.persist_schema {
            println!(
                "Updated {} with inferred config_schema ({:?})",
                manifest_path.display(),
                config.source
            );
        }
        if let Some(flows) = flow_result {
            if flows.default_written || flows.custom_written {
                println!(
                    "Flows scaffolded (default: {}, custom: {})",
                    flows.default_written, flows.custom_written
                );
            } else {
                println!("Flows left unchanged");
            }
        } else {
            println!("Flow scaffolding skipped (--no-flow)");
        }
    }

    Ok(())
}

fn build_wasm(manifest_dir: &Path, cargo_bin: &Path) -> Result<()> {
    println!(
        "Running cargo build via {} in {}",
        cargo_bin.display(),
        manifest_dir.display()
    );
    let status = Command::new(cargo_bin)
        .arg("build")
        .arg("--target")
        .arg("wasm32-wasip2")
        .arg("--release")
        .current_dir(manifest_dir)
        .status()
        .with_context(|| format!("failed to run cargo build via {}", cargo_bin.display()))?;

    if !status.success() {
        bail!(
            "cargo build --target wasm32-wasip2 --release failed with status {}",
            status
        );
    }
    Ok(())
}

fn update_manifest_hashes(
    manifest_dir: &Path,
    manifest: &mut JsonValue,
) -> Result<(PathBuf, String)> {
    let artifact_path = resolve_wasm_path(manifest_dir, manifest)?;
    let wasm_bytes = fs::read(&artifact_path)
        .with_context(|| format!("failed to read wasm at {}", artifact_path.display()))?;
    let digest = blake3::hash(&wasm_bytes).to_hex().to_string();

    manifest["artifacts"]["component_wasm"] =
        JsonValue::String(path_string_relative(manifest_dir, &artifact_path)?);
    manifest["hashes"]["component_wasm"] = JsonValue::String(format!("blake3:{digest}"));

    Ok((artifact_path, format!("blake3:{digest}")))
}

fn path_string_relative(base: &Path, target: &Path) -> Result<String> {
    let rel = pathdiff::diff_paths(target, base).unwrap_or_else(|| target.to_path_buf());
    rel.to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("failed to stringify path {}", target.display()))
}

fn resolve_wasm_path(manifest_dir: &Path, manifest: &JsonValue) -> Result<PathBuf> {
    let manifest_root = manifest_dir
        .canonicalize()
        .with_context(|| format!("failed to canonicalize {}", manifest_dir.display()))?;
    let candidate = manifest
        .get("artifacts")
        .and_then(|a| a.get("component_wasm"))
        .and_then(|v| v.as_str())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let raw_name = manifest
                .get("name")
                .and_then(|v| v.as_str())
                .or_else(|| manifest.get("id").and_then(|v| v.as_str()))
                .unwrap_or("component");
            let sanitized = raw_name.replace(['-', '.'], "_");
            manifest_dir.join(format!("target/wasm32-wasip2/release/{sanitized}.wasm"))
        });
    let normalized = normalize_under_root(&manifest_root, &candidate).or_else(|_| {
        if candidate.is_absolute() {
            candidate
                .canonicalize()
                .with_context(|| format!("failed to canonicalize {}", candidate.display()))
        } else {
            normalize_under_root(&manifest_root, &candidate)
        }
    })?;
    Ok(normalized)
}

fn write_manifest(manifest_path: &Path, manifest: &JsonValue) -> Result<()> {
    let formatted = serde_json::to_string_pretty(manifest)?;
    fs::write(manifest_path, formatted + "\n")
        .with_context(|| format!("failed to write {}", manifest_path.display()))
}
