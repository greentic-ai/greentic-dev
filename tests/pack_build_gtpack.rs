use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use std::path::{Path, PathBuf};

fn write_pack(root: &Path, pack_id: &str, wasm_src: &Path) -> PathBuf {
    let dir = root.join(pack_id);
    fs::create_dir_all(dir.join("flows")).unwrap();
    fs::create_dir_all(dir.join("components")).unwrap();
    fs::copy(wasm_src, dir.join("components/shared_comp.wasm")).unwrap();

    fs::write(
        dir.join("flows/main.ygtc"),
        "id: main
title: Minimal
type: messaging
start: start

nodes:
  start:
    templating.handlebars:
      text: \"hi\"
    routing:
      - out: true
",
    )
    .unwrap();

    fs::write(
        dir.join("pack.yaml"),
        format!(
            "pack_id: {pack_id}
version: 0.1.0
kind: application
publisher: Greentic

components:
  - id: \"{pack_id}.shared\"
    version: \"0.1.0\"
    world: \"greentic:component/stub\"
    supports: [\"messaging\"]
    profiles: {{ default: \"default\", supported: [\"default\"] }}
    capabilities: {{ wasi: {{}}, host: {{}} }}
    wasm: \"components/shared_comp.wasm\"

flows:
  - id: main
    file: flows/main.ygtc
    tags: [default]
    entrypoints: [default]

dependencies: []

assets: []
"
        ),
    )
    .unwrap();

    dir
}

#[test]
fn pack_build_emits_gtpack_for_each_pack() {
    let tmp = tempfile::tempdir().unwrap();
    let shared_wasm = tmp.path().join("shared_comp.wasm");
    fs::write(&shared_wasm, b"00").unwrap();

    let pack_a = write_pack(tmp.path(), "pack-a", &shared_wasm);
    let pack_b = write_pack(tmp.path(), "pack-b", &shared_wasm);

    for pack_dir in [&pack_a, &pack_b] {
        let expected = pack_dir.join("target").join(format!(
            "{}.gtpack",
            pack_dir.file_name().unwrap().to_string_lossy()
        ));

        let mut cmd = cargo_bin_cmd!("greentic-dev");
        cmd.env("GREENTIC_DEV_OFFLINE", "1")
            .current_dir(pack_dir)
            .arg("pack")
            .arg("build")
            .arg("--in")
            .arg(".");
        cmd.assert().success();

        assert!(
            expected.exists(),
            "expected gtpack at {}, command: {:?}",
            expected.display(),
            cmd
        );
    }
}
