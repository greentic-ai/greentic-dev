#![cfg(feature = "cli")]

use assert_cmd::prelude::*;
use assert_fs::TempDir;
use insta::assert_json_snapshot;
use serde_json::Value;
use std::fs;
use std::process::Command;

#[path = "snapshot_util.rs"]
mod snapshot_util;

use snapshot_util::normalize_value_paths;

fn run_templates_json(envs: &[(&str, &std::path::Path)]) -> Value {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("greentic-component"));
    cmd.arg("templates").arg("--json");
    for (key, value) in envs {
        cmd.env(key, value);
    }
    let assert = cmd.assert().success();
    serde_json::from_slice(&assert.get_output().stdout).expect("json")
}

#[test]
fn templates_only_builtin_json() {
    let temp_home = TempDir::new().expect("temp dir");
    let mut value = run_templates_json(&[("HOME", temp_home.path())]);
    normalize_value_paths(&mut value);
    assert_json_snapshot!("templates_only_builtin_json", value);
}

#[test]
fn templates_include_user_metadata() {
    let temp_home = TempDir::new().expect("temp dir");
    let template_root = temp_home.path().join("templates");
    let user_template = template_root.join("custom-template");
    fs::create_dir_all(&user_template).expect("user template dir");
    fs::write(
        user_template.join("template.json"),
        r#"{
            "id": "user-template",
            "description": "User provided template",
            "tags": ["user", "testing"]
        }"#,
    )
    .expect("template json");
    let mut value = run_templates_json(&[
        ("HOME", temp_home.path()),
        ("GREENTIC_TEMPLATE_ROOT", template_root.as_path()),
    ]);
    if let Value::Array(entries) = &mut value {
        for entry in entries {
            if entry.get("location").and_then(|loc| loc.as_str()) == Some("user") {
                entry["path"] = Value::String("<user>".into());
            }
        }
    }
    normalize_value_paths(&mut value);
    assert_json_snapshot!("templates_with_user_metadata_json", value);
}
