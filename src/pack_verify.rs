use std::path::Path;

use anyhow::Result;
use greentic_pack::reader::{PackVerifyResult, SigningPolicy, open_pack};
use serde_json::json;

#[derive(Debug, Clone, Copy)]
pub enum VerifyPolicy {
    Strict,
    DevOk,
}

impl From<VerifyPolicy> for SigningPolicy {
    fn from(policy: VerifyPolicy) -> Self {
        match policy {
            VerifyPolicy::Strict => SigningPolicy::Strict,
            VerifyPolicy::DevOk => SigningPolicy::DevOk,
        }
    }
}

pub fn run(pack_path: &Path, policy: VerifyPolicy, emit_json: bool) -> Result<()> {
    let load = open_pack(pack_path, policy.into()).map_err(|err: PackVerifyResult| {
        anyhow::anyhow!("pack verification failed: {}", err.message)
    })?;

    if emit_json {
        let doc = json!({
            "manifest": load.manifest,
            "report": {
                "signature_ok": load.report.signature_ok,
                "sbom_ok": load.report.sbom_ok,
                "warnings": load.report.warnings,
            },
            "sbom": load.sbom,
        });
        println!("{}", serde_json::to_string_pretty(&doc)?);
    } else {
        println!("✓ Pack verified: {}", pack_path.display());
        if !load.report.warnings.is_empty() {
            println!("Warnings:");
            for warning in &load.report.warnings {
                println!("- {warning}");
            }
        }
    }

    Ok(())
}
