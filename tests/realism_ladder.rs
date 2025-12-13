mod support;

use anyhow::{Context, Result};
use greentic_dev::cli::FlowAddStepArgs;
use greentic_dev::flow_cmd::run_add_step;
use greentic_dev::pack_verify::{self, VerifyPolicy};
use support::{
    WORKDIR_LOCK, Workspace, build_pack, compute_blake3_hex, copy_fixture_component, diag,
    load_gtpack, write_pack_flow,
};

#[test]
fn pack_realism_l0_minimal_builds() -> Result<()> {
    let workspace = Workspace::new("realism-l0")?;
    let component_dir = copy_fixture_component(&workspace, false)?;
    let flow_path = write_pack_flow(&workspace, "hello-flow")?;

    let pack_path = build_pack(
        &workspace,
        &flow_path,
        component_dir.parent().expect("component root"),
    )?;
    assert!(
        pack_path.exists(),
        "gtpack should be created at {}",
        pack_path.display()
    );

    let (manifest, files) = load_gtpack(&pack_path).context("load gtpack")?;
    assert!(
        !manifest.meta.pack_id.trim().is_empty(),
        "pack id should be set"
    );
    assert!(
        !manifest.components.is_empty(),
        "pack should embed at least one component"
    );

    let first = &manifest.components[0];
    let manifest_path = first
        .manifest_file
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("component manifest path missing from pack"))?;
    let bytes = files
        .get(manifest_path)
        .ok_or_else(|| anyhow::anyhow!("component manifest bytes missing from pack"))?;
    let manifest_json: serde_json::Value = serde_json::from_slice(bytes)?;
    let dev_flow = manifest_json
        .get("dev_flows")
        .and_then(|m| m.get("default"))
        .ok_or_else(|| anyhow::anyhow!("missing dev_flows.default in component manifest"))?;
    assert!(
        dev_flow.get("graph").and_then(|g| g.as_object()).is_some(),
        "dev_flows.default.graph should be an object"
    );

    Ok(())
}

#[test]
fn pack_realism_l0_missing_default_flow_error() -> Result<()> {
    let workspace = Workspace::new("realism-l0-missing")?;
    let component_dir = copy_fixture_component(&workspace, true)?;

    let _guard = WORKDIR_LOCK.lock().unwrap();
    let prev = std::env::current_dir()?;
    std::env::set_current_dir(&workspace.root)?;

    let err = run_add_step(FlowAddStepArgs {
        flow_id: "demo".into(),
        coordinate: Some(component_dir.to_string_lossy().to_string()),
        profile: None,
        mode: None,
        after: Some("start".into()),
        flow: "default".into(),
        manifest: Some(component_dir.join("component.manifest.json")),
    })
    .expect_err("expected missing dev_flow error");

    std::env::set_current_dir(prev)?;
    let expected = "Flow 'default' is missing from manifest.dev_flows. Run `greentic-component flow update` to regenerate config flows.";
    assert_eq!(err.to_string(), expected);
    Ok(())
}

#[test]
fn pack_realism_l1_artifacts_and_hashes_consistent() -> Result<()> {
    let workspace = Workspace::new("realism-l1")?;
    let component_dir = copy_fixture_component(&workspace, false)?;
    let flow_path = write_pack_flow(&workspace, "hello-flow")?;

    let pack_path = build_pack(
        &workspace,
        &flow_path,
        component_dir.parent().expect("component root"),
    )?;
    let (manifest, artifacts) = load_gtpack(&pack_path)?;

    let component = manifest
        .components
        .iter()
        .find(|c| c.name == "dev.greentic.echo")
        .ok_or_else(|| anyhow::anyhow!("component dev.greentic.echo missing from pack"))?;
    let expected_hash = &component.hash_blake3;

    let wasm_entry = &component.file_wasm;
    let bytes = artifacts.get(wasm_entry.as_str()).ok_or_else(|| {
        diag(
            "pack_realism_l1_artifacts_and_hashes_consistent",
            "gtpack lookup",
            &workspace,
            &format!("missing {}", wasm_entry),
        );
        anyhow::anyhow!("component artifact missing from gtpack")
    })?;
    let actual_hash = compute_blake3_hex(bytes);
    assert_eq!(
        actual_hash,
        expected_hash.as_str(),
        "component_wasm hash should match packed bytes"
    );

    pack_verify::run(&pack_path, VerifyPolicy::DevOk, false)
        .context("pack verify should succeed")?;
    Ok(())
}
