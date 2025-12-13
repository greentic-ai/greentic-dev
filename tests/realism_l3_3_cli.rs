mod support;

use std::process::Command;

use anyhow::{Context, Result};
use serde_json::Value as JsonValue;
use support::{Workspace, build_pack, copy_fixture_component, diag_with_owner, write_pack_flow};

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

fn parse_json(stdout: &str) -> Result<JsonValue> {
    serde_json::from_str(stdout.trim()).context("stdout is not valid JSON")
}

fn run_cli(pack_path: &std::path::Path, entry: &str, input: &str) -> Result<(i32, String, String)> {
    let bin = resolve_bin()?;
    let output = Command::new(bin)
        .arg("pack")
        .arg("run")
        .arg("--offline")
        .arg("--json")
        .arg("--entry")
        .arg(entry)
        .arg("-p")
        .arg(pack_path)
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

#[test]
fn pack_realism_l3_3_cli_real_runtime_executes_wasm_offline() -> Result<()> {
    let workspace = Workspace::new("realism-l3.3-runtime")?;
    let component_dir = copy_fixture_component(&workspace, false)?;
    let flow_path = write_pack_flow(&workspace, "hello-flow")?;
    let pack_path = build_pack(
        &workspace,
        &flow_path,
        component_dir.parent().expect("component root"),
    )?;

    let (code, stdout, stderr) = run_cli(&pack_path, "hello-flow", r#"{"query":"hello"}"#)?;
    if code != 0 {
        diag_with_owner(
            "pack_realism_l3_3_cli_real_runtime_executes_wasm_offline",
            "execute",
            &workspace,
            &format!("exit {code}, stderr: {stderr}"),
            "greentic-dev",
        );
        anyhow::bail!("cli exit code {code}");
    }
    assert!(
        !stderr.contains("panicked at"),
        "stderr contains panic: {stderr}"
    );

    let doc = parse_json(&stdout)?;
    assert_eq!(
        doc.get("exec_mode").and_then(|v| v.as_str()),
        Some("runtime")
    );
    let status = doc.get("status").and_then(|v| v.as_str()).unwrap_or("");
    assert_eq!(status, "Success");

    // Node summaries show runtime execution; expect at least one node.
    let node_summaries = doc
        .get("node_summaries")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    assert!(!node_summaries.is_empty(), "node_summaries missing");
    // output is not present in runner output; rely on node summaries to confirm execution
    let first_status = node_summaries
        .first()
        .and_then(|n| n.get("status"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(
        first_status.eq_ignore_ascii_case("ok"),
        "expected Ok status in node summaries"
    );
    Ok(())
}
