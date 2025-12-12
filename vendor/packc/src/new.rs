#![forbid(unsafe_code)]

use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
pub struct NewArgs {
    /// Directory to create the pack in
    #[arg(long = "dir", value_name = "DIR")]
    pub dir: PathBuf,
    /// Pack id to use
    #[arg(value_name = "PACK_ID")]
    pub pack_id: String,
}

pub fn handle(args: NewArgs, json: bool) -> Result<()> {
    let root = args.dir.canonicalize().unwrap_or_else(|_| args.dir.clone());
    fs::create_dir_all(&root)?;

    write_pack_yaml(&root, &args.pack_id)?;
    write_flow(&root)?;
    write_stub_component(&root)?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "status": "ok",
                "pack_dir": root,
            }))?
        );
    } else {
        println!("created pack at {}", root.display());
    }

    Ok(())
}

fn write_pack_yaml(root: &Path, pack_id: &str) -> Result<()> {
    let pack_yaml = format!(
        r#"pack_id: {pack_id}
version: 0.1.0
kind: application
publisher: Greentic

components:
  - id: "{pack_id}.component"
    version: "0.1.0"
    world: "greentic:component/stub"
    supports: ["messaging"]
    profiles:
      default: "default"
      supported: ["default"]
    capabilities:
      wasi: {{}}
      host: {{}}
    wasm: "components/stub.wasm"

flows:
  - id: main
    file: flows/main.ygtc
    tags: [default]
    entrypoints: [default]

dependencies: []

assets: []
"#
    );
    let path = root.join("pack.yaml");
    fs::write(&path, pack_yaml).with_context(|| format!("failed to write {}", path.display()))
}

fn write_flow(root: &Path) -> Result<()> {
    let flows_dir = root.join("flows");
    fs::create_dir_all(&flows_dir)?;
    let flow_path = flows_dir.join("main.ygtc");
    const FLOW: &str = r#"id: main
title: Welcome
description: Minimal starter flow
type: messaging
start: start

nodes:
  start:
    templating.handlebars:
      text: "Hello from greentic-pack starter!"
    routing:
      - out: true
"#;
    fs::write(&flow_path, FLOW).with_context(|| format!("failed to write {}", flow_path.display()))
}

fn write_stub_component(root: &Path) -> Result<()> {
    let components_dir = root.join("components");
    fs::create_dir_all(&components_dir)?;
    let path = components_dir.join("stub.wasm");
    if !path.exists() {
        fs::write(&path, &[0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00][..])
            .with_context(|| format!("failed to write {}", path.display()))?;
    }
    Ok(())
}
