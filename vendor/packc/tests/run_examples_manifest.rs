use assert_cmd::prelude::*;
use std::process::Command;

fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

#[test]
fn build_all_examples_manifest_only() {
    let packs = [
        "examples/weather-demo",
        "examples/qa-demo",
        "examples/billing-demo",
        "examples/search-demo",
        "examples/reco-demo",
    ];

    for pack in packs {
        let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("packc"));
        cmd.current_dir(workspace_root());
        cmd.args(["build", "--in", pack, "--dry-run", "--log", "warn"]);
        cmd.assert().success();
    }
}
