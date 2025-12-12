#![cfg(feature = "cli")]

use assert_cmd::prelude::*;
use insta::assert_snapshot;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[path = "snapshot_util.rs"]
mod snapshot_util;

use snapshot_util::normalize_text;

#[test]
fn scaffold_rust_wasi_template() {
    let temp = TempDir::new().expect("temp dir");
    let component_dir = temp.path().join("demo-component");
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("greentic-component"));
    cmd.arg("new")
        .arg("--name")
        .arg("demo-component")
        .arg("--org")
        .arg("ai.greentic")
        .arg("--path")
        .arg(&component_dir)
        .arg("--no-check")
        .env("HOME", temp.path())
        .env("GREENTIC_TEMPLATE_YEAR", "2030")
        .env("GREENTIC_TEMPLATE_ROOT", temp.path().join("templates"))
        .env("GIT_AUTHOR_NAME", "Greentic Labs")
        .env("GIT_AUTHOR_EMAIL", "greentic-labs@example.com")
        .env("GIT_COMMITTER_NAME", "Greentic Labs")
        .env("GIT_COMMITTER_EMAIL", "greentic-labs@example.com")
        .env_remove("USER")
        .env_remove("USERNAME");
    cmd.assert().success();

    let cargo = fs::read_to_string(component_dir.join("Cargo.toml")).expect("Cargo.toml");
    let manifest =
        fs::read_to_string(component_dir.join("component.manifest.json")).expect("manifest");

    assert_snapshot!("scaffold_cargo_toml", normalize_text(cargo.trim()));
    assert_snapshot!("scaffold_manifest", normalize_text(manifest.trim()));
    let wit_dir = component_dir.join("wit");
    assert!(
        wit_dir.exists(),
        "template should emit WIT files for config inference"
    );
    assert!(
        wit_dir.join("world.wit").exists(),
        "world.wit should be scaffolded"
    );

    assert!(
        component_dir.join(".git").exists(),
        "post-render hook should initialize git"
    );
    let rev_parse = Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .current_dir(&component_dir)
        .output()
        .expect("git rev-parse");
    assert!(rev_parse.status.success(), "git rev-parse should succeed");
    let status = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .current_dir(&component_dir)
        .output()
        .expect("git status");
    assert!(
        status.status.success(),
        "git status should succeed after initial commit"
    );
    assert!(
        String::from_utf8_lossy(&status.stdout).trim().is_empty(),
        "repository should be clean after initial commit"
    );
}
