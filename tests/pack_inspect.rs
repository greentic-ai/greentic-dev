use std::fs;
use std::process::Command;

use anyhow::Result;
use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use tempfile::tempdir;

fn preferred_subcommand() -> Option<&'static str> {
    let help = Command::new("greentic-pack").arg("--help").output().ok()?;
    let text = String::from_utf8_lossy(&help.stdout);
    if text.contains("doctor") {
        Some("doctor")
    } else if text.contains("inspect") {
        Some("inspect")
    } else {
        None
    }
}

// Regression guard: greentic-pack-built gtpack artifacts should be accepted by greentic-dev inspect/doctor.
#[test]
fn greentic_pack_gtpack_is_inspectable() -> Result<()> {
    let subcmd = match preferred_subcommand() {
        Some(cmd) => cmd,
        None => {
            eprintln!("skipping: greentic-pack has neither doctor nor inspect");
            return Ok(());
        }
    };

    let temp = tempdir()?;
    let pack_dir = temp.path().join("demo-pack");
    let gtpack_path = pack_dir.join("pack.gtpack");

    let new_status = Command::new("greentic-pack")
        .args(["new", "--dir", pack_dir.to_str().unwrap(), "demo.test"])
        .status()
        .expect("failed to spawn greentic-pack new");
    assert!(new_status.success(), "greentic-pack new failed");

    let build_status = Command::new("greentic-pack")
        .args([
            "build",
            "--in",
            pack_dir.to_str().unwrap(),
            "--gtpack-out",
            gtpack_path.to_str().unwrap(),
            "--offline",
            "--allow-oci-tags",
        ])
        .status()
        .expect("failed to spawn greentic-pack build");
    assert!(build_status.success(), "greentic-pack build failed");

    assert!(fs::metadata(&gtpack_path).is_ok(), "gtpack not written");

    let mut cmd = cargo_bin_cmd!("greentic-dev");
    cmd.args(["pack", subcmd]).arg(&gtpack_path);
    let assert = cmd.assert().success();

    // Accept either structured JSON (older inspect) or human-readable doctor output.
    if subcmd == "inspect" {
        assert.stdout(
            contains("OK")
                .or(contains("\"status\""))
                .or(contains("\"ok\":true")),
        );
    }

    Ok(())
}
