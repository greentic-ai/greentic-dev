mod support;

use anyhow::Result;
use serde_json::json;
use support::exec::execute_flow_from_pack;
use support::l3::build_l3_pack;
use support::{Workspace, diag_with_owner};

#[test]
fn pack_realism_l3_executes_success_path() -> Result<()> {
    let workspace = Workspace::new("realism-l3-success")?;
    let pack_bytes = build_l3_pack()?;
    let result = execute_flow_from_pack(&pack_bytes, "default", json!({ "query": "hello" }))
        .map_err(|err| {
            diag_with_owner(
                "pack_realism_l3_executes_success_path",
                "execute",
                &workspace,
                &format!("err: {err}"),
                "greentic-flow",
            );
            err
        })?;
    assert_eq!(
        result.output.get("answer").and_then(|v| v.as_str()),
        Some("Result: fixed")
    );
    assert_eq!(result.trace.len(), 3);
    assert_eq!(result.trace[1].component, "component.tool.fixed");
    Ok(())
}

#[test]
fn pack_realism_l3_executes_error_path_and_err_map_applies() -> Result<()> {
    let workspace = Workspace::new("realism-l3-error")?;
    let pack_bytes = build_l3_pack()?;
    let result =
        execute_flow_from_pack(&pack_bytes, "default", json!({ "fail": true })).map_err(|err| {
            diag_with_owner(
                "pack_realism_l3_executes_error_path_and_err_map_applies",
                "execute",
                &workspace,
                &format!("err: {err}"),
                "greentic-flow",
            );
            err
        })?;
    assert!(
        result
            .output
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .contains("friendly"),
        "expected friendly error message"
    );
    assert_eq!(result.trace.len(), 3);
    assert_eq!(result.trace[1].status, "error");
    Ok(())
}

#[test]
fn pack_realism_l3_output_shape_stable() -> Result<()> {
    let pack_bytes = build_l3_pack()?;
    let result = execute_flow_from_pack(&pack_bytes, "default", json!({ "query": "hi" }))?;
    let answer = result
        .output
        .get("answer")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(
        answer.starts_with("Result:"),
        "answer should be templated string"
    );
    assert!(result.output.get("input").is_some());
    Ok(())
}
