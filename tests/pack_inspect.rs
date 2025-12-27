use std::fs;

use anyhow::Result;
use assert_cmd::cargo::cargo_bin_cmd;
use tempfile::tempdir;

// Regression guard: packc-built gtpack artifacts should be accepted by greentic-dev inspect.
#[test]
fn packc_gtpack_is_inspectable() -> Result<()> {
    let temp = tempdir()?;
    let pack_dir = temp.path().join("demo-pack");
    let gtpack_path = pack_dir.join("pack.gtpack");

    cargo_bin_cmd!("packc")
        .arg("new")
        .arg("--dir")
        .arg(&pack_dir)
        .arg("demo.test")
        .assert()
        .success();

    cargo_bin_cmd!("packc")
        .arg("build")
        .arg("--in")
        .arg(&pack_dir)
        .arg("--gtpack-out")
        .arg(&gtpack_path)
        .arg("--offline")
        .assert()
        .success();

    assert!(fs::metadata(&gtpack_path).is_ok(), "gtpack not written");

    cargo_bin_cmd!("greentic-dev")
        .arg("pack")
        .arg("inspect")
        .arg(&gtpack_path)
        .assert()
        .success();

    Ok(())
}
