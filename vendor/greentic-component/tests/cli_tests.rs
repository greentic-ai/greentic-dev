#![cfg(all(feature = "cli", feature = "prepare"))]

#[path = "support/mod.rs"]
mod support;

use greentic_component::scaffold::deps::DependencyMode;
use greentic_component::scaffold::engine::{DEFAULT_WIT_WORLD, ScaffoldEngine, ScaffoldRequest};
use predicates::prelude::*;
use serde_json::Value;
use support::TestComponent;

const TEST_WIT: &str = r#"
package greentic:component@0.1.0;
world node {
    export describe: func();
}
"#;

#[test]
fn inspect_outputs_json() {
    let component = TestComponent::new(TEST_WIT, &["describe"]);
    let manifest_path = component.manifest_path.to_str().unwrap();
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("component-inspect");
    cmd.arg(manifest_path)
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"manifest\""));
}

#[test]
fn doctor_reports_success() {
    let component = TestComponent::new(TEST_WIT, &["describe"]);
    let manifest_path = component.manifest_path.to_str().unwrap();
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("component-doctor");
    cmd.arg(manifest_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("manifest schema: ok"));
}

#[test]
fn doctor_detects_scaffold_directory() {
    let temp = tempfile::TempDir::new().unwrap();
    let root = temp.path().join("demo-detect");
    let engine = ScaffoldEngine::new();
    let request = ScaffoldRequest {
        name: "demo-detect".into(),
        path: root.clone(),
        template_id: "rust-wasi-p2-min".into(),
        org: "ai.greentic".into(),
        version: "0.1.0".into(),
        license: "MIT".into(),
        wit_world: DEFAULT_WIT_WORLD.into(),
        non_interactive: true,
        year_override: Some(2030),
        dependency_mode: DependencyMode::Local,
    };
    engine.scaffold(request).unwrap();
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("component-doctor");
    cmd.arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Detected Greentic scaffold"));
}

#[test]
fn new_outputs_template_metadata_in_json() {
    let temp = tempfile::TempDir::new().unwrap();
    let project = temp.path().join("json-demo");
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("greentic-component");
    let assert = cmd
        .arg("new")
        .arg("--name")
        .arg("json-demo")
        .arg("--org")
        .arg("ai.greentic")
        .arg("--path")
        .arg(&project)
        .arg("--no-check")
        .arg("--no-git")
        .arg("--json")
        .env("HOME", temp.path())
        .env("GREENTIC_TEMPLATE_YEAR", "2030")
        .assert()
        .success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).expect("utf8 stdout");
    let value: Value = serde_json::from_str(&output).expect("json");
    assert_eq!(
        value["scaffold"]["template"].as_str().unwrap(),
        "rust-wasi-p2-min"
    );
    assert_eq!(
        value["scaffold"]["template_description"].as_str().unwrap(),
        "Minimal Rust + WASI-P2 component starter"
    );
    assert_eq!(
        value["post_init"]["git"]["status"].as_str().unwrap(),
        "skipped"
    );
    assert!(
        value["post_init"]["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["stage"] == "git-init")
    );
}
