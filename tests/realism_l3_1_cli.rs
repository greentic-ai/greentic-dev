mod support;

use std::process::Command;

use anyhow::{Context, Result};
use serde_json::Value as JsonValue;
use support::l3::build_l3_pack;
use support::{Workspace, diag_with_owner};

fn resolve_bin() -> Result<std::path::PathBuf> {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_greentic-dev") {
        return Ok(std::path::PathBuf::from(path));
    }
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_greentic_dev") {
        return Ok(std::path::PathBuf::from(path));
    }
    let current = std::env::current_exe().context("current_exe")?;
    let candidate = current
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("greentic-dev"))
        .ok_or_else(|| anyhow::anyhow!("cannot resolve greentic-dev binary"))?;
    Ok(candidate)
}

fn run_cli(pack_path: &std::path::Path, input: &str) -> Result<(i32, String, String)> {
    let bin = resolve_bin()?;
    let output = Command::new(bin)
        .arg("pack")
        .arg("run")
        .arg("-p")
        .arg(pack_path)
        .arg("--json")
        .arg("--mock-exec")
        .arg("--offline")
        .arg("--input")
        .arg(input)
        .env("HTTP_PROXY", "")
        .env("HTTPS_PROXY", "")
        .env("ALL_PROXY", "")
        .env("NO_PROXY", "*")
        .output()
        .context("failed to run CLI")?;
    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    Ok((code, stdout, stderr))
}

fn parse_json(stdout: &str) -> Result<JsonValue> {
    serde_json::from_str(stdout.trim()).context("stdout is not valid JSON")
}

#[test]
fn pack_realism_l3_1_cli_executes_success_path() -> Result<()> {
    let workspace = Workspace::new("realism-l3.1-success")?;
    let pack_bytes = build_l3_pack()?;
    let pack_path = workspace.root.join("l3.gtpack");
    std::fs::create_dir_all(pack_path.parent().unwrap()).unwrap();
    std::fs::write(&pack_path, &pack_bytes)?;

    let (code, stdout, stderr) = run_cli(&pack_path, r#"{"query":"hello"}"#)?;
    if code != 0 {
        diag_with_owner(
            "pack_realism_l3_1_cli_executes_success_path",
            "execute",
            &workspace,
            &format!("exit {code}, stderr: {stderr}"),
            "greentic-dev",
        );
        anyhow::bail!("cli exited with {}", code);
    }
    let doc = parse_json(&stdout)?;
    let output = doc
        .get("output")
        .and_then(|v| v.get("answer"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(output.contains("Result: fixed"));
    let empty = Vec::new();
    let trace = doc
        .get("trace")
        .and_then(|v| v.as_array())
        .unwrap_or(&empty);
    assert!(
        trace.len() >= 2,
        "expected trace entries, got {}",
        trace.len()
    );
    Ok(())
}

#[test]
fn pack_realism_l3_1_cli_executes_error_path() -> Result<()> {
    let workspace = Workspace::new("realism-l3.1-error")?;
    let pack_bytes = build_l3_pack()?;
    let pack_path = workspace.root.join("l3.gtpack");
    std::fs::create_dir_all(pack_path.parent().unwrap()).unwrap();
    std::fs::write(&pack_path, &pack_bytes)?;

    let (code, stdout, _stderr) = run_cli(&pack_path, r#"{"fail":true}"#)?;
    let doc = parse_json(&stdout)?;
    let status = doc.get("status").and_then(|v| v.as_str()).unwrap_or("");
    assert!(
        status == "error" || code != 0,
        "expected error status or exit, got status {status}, code {code}"
    );
    let message = doc
        .get("output")
        .and_then(|v| v.get("message"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(
        message.contains("friendly"),
        "expected friendly message in output"
    );
    Ok(())
}
