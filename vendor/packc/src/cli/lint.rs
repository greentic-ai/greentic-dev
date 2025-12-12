#![forbid(unsafe_code)]

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use greentic_flow::compile_ygtc_str;
use tracing::info;

use crate::config::load_pack_config;

#[derive(Debug, Parser)]
pub struct LintArgs {
    /// Root directory of the pack (must contain pack.yaml)
    #[arg(long = "in", value_name = "DIR")]
    pub input: PathBuf,
}

pub fn handle(args: LintArgs, json: bool) -> Result<()> {
    let pack_dir = normalize(args.input);
    info!(path = %pack_dir.display(), "linting pack");

    let cfg = load_pack_config(&pack_dir)?;

    let mut compiled = 0usize;
    for flow in &cfg.flows {
        let src = std::fs::read_to_string(&flow.file)?;
        compile_ygtc_str(&src)?;
        compiled += 1;
    }

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "status": "ok",
                "pack_id": cfg.pack_id,
                "version": cfg.version,
                "flows": compiled,
                "components": cfg.components.len(),
                "dependencies": cfg.dependencies.len(),
            }))?
        );
    } else {
        println!(
            "lint ok\n  pack: {}@{}\n  flows: {}\n  components: {}\n  dependencies: {}",
            cfg.pack_id,
            cfg.version,
            compiled,
            cfg.components.len(),
            cfg.dependencies.len()
        );
    }

    Ok(())
}

fn normalize(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    }
}
