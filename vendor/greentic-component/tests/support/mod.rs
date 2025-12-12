#![cfg(feature = "prepare")]

use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};

use blake3::Hasher;
use greentic_component::manifest::{ComponentManifest, parse_manifest};
use serde_json::{self, json};
use tempfile::TempDir;
use wasm_encoder::{
    CodeSection, CustomSection, ExportKind, ExportSection, Function, FunctionSection, Instruction,
    Module, TypeSection,
};
use wit_component::StringEncoding;
use wit_component::metadata;
use wit_parser::{Resolve, WorldId};

const WASI_MARKER: &str = "wasm32-wasip2";

#[allow(dead_code)]
pub struct TestComponent {
    pub dir: TempDir,
    pub wasm_path: PathBuf,
    pub manifest_path: PathBuf,
    pub manifest: ComponentManifest,
    pub world: String,
}

impl TestComponent {
    pub fn new(world_src: &str, funcs: &[&str]) -> Self {
        let dir = TempDir::new().expect("tempdir");
        let (wasm_bytes, world_ref) = build_module(world_src, funcs);
        let wasm_path = dir.path().join("component.wasm");
        fs::write(&wasm_path, &wasm_bytes).expect("write wasm");

        let hash = blake3_hash(&wasm_bytes);
        let world_value = world_ref.clone();
        let manifest_json = json!({
            "id": "com.greentic.test.component",
            "name": "Test Component",
            "version": "0.1.0",
            "world": world_value,
            "describe_export": "describe",
            "supports": ["messaging"],
            "profiles": {
                "default": "stateless",
                "supported": ["stateless"]
            },
            "capabilities": {
                "wasi": {
                    "filesystem": {
                        "mode": "none",
                        "mounts": []
                    },
                    "random": true,
                    "clocks": true
                },
                "host": {
                    "messaging": {
                        "inbound": true,
                        "outbound": true
                    },
                    "telemetry": {
                        "scope": "tenant"
                    }
                }
            },
            "limits": {
                "memory_mb": 64,
                "wall_time_ms": 1000,
                "fuel": 10,
                "files": 2
            },
            "telemetry": {
                "span_prefix": "test.component",
                "attributes": {
                    "component": "test"
                },
                "emit_node_spans": true
            },
            "provenance": {
                "builder": "greentic-dev",
                "git_commit": "abcdef1",
                "toolchain": "rustc",
                "built_at_utc": "2024-01-01T00:00:00Z"
            },
            "artifacts": {
                "component_wasm": "component.wasm"
            },
            "hashes": {
                "component_wasm": hash
            }
        });

        let manifest_path = dir.path().join("component.manifest.json");
        fs::write(
            &manifest_path,
            serde_json::to_string_pretty(&manifest_json).unwrap(),
        )
        .expect("manifest");
        let manifest = parse_manifest(&manifest_json.to_string()).expect("manifest parse");

        TestComponent {
            dir,
            wasm_path,
            manifest_path,
            manifest,
            world: world_ref,
        }
    }
}

fn build_module(world_src: &str, funcs: &[&str]) -> (Vec<u8>, String) {
    let mut resolve = Resolve::default();
    let pkg = resolve.push_str("test.wit", world_src).expect("push wit");
    let world = resolve
        .select_world(&[pkg], Some("node"))
        .expect("world lookup");
    let label = world_label(&resolve, world);
    let metadata =
        metadata::encode(&resolve, world, StringEncoding::UTF8, None).expect("metadata encode");

    let mut module = Module::new();
    let mut types = TypeSection::new();
    types.ty().function([], []);
    module.section(&types);

    let mut funcs_section = FunctionSection::new();
    for _ in funcs {
        funcs_section.function(0);
    }
    module.section(&funcs_section);

    let mut exports = ExportSection::new();
    for (idx, name) in funcs.iter().enumerate() {
        exports.export(name, ExportKind::Func, idx as u32);
    }
    module.section(&exports);

    let mut code = CodeSection::new();
    for _ in funcs {
        let mut body = Function::new([]);
        body.instruction(&Instruction::End);
        code.function(&body);
    }
    module.section(&code);

    module.section(&CustomSection {
        name: "component-type".into(),
        data: Cow::Borrowed(&metadata),
    });

    module.section(&CustomSection {
        name: "producers".into(),
        data: Cow::Borrowed(WASI_MARKER.as_bytes()),
    });

    let wasm = module.finish();
    let observed = detect_world(&wasm).unwrap_or(label);
    (wasm, observed)
}

fn blake3_hash(bytes: &[u8]) -> String {
    let mut hasher = Hasher::new();
    hasher.update(bytes);
    format!("blake3:{}", hex::encode(hasher.finalize().as_bytes()))
}

#[allow(dead_code)]
pub fn write_embedded_payload(root: &Path, payload: &serde_json::Value) -> PathBuf {
    let dir = root.join("schemas").join("v1");
    fs::create_dir_all(&dir).expect("schema dir");
    let path = dir.join("payload.json");
    fs::write(&path, serde_json::to_string_pretty(payload).unwrap()).expect("payload");
    path
}

fn world_label(resolve: &Resolve, world_id: WorldId) -> String {
    let world = &resolve.worlds[world_id];
    if let Some(pkg_id) = world.package {
        let pkg = &resolve.packages[pkg_id];
        if let Some(version) = &pkg.name.version {
            format!(
                "{}:{}/{}@{}",
                pkg.name.namespace, pkg.name.name, world.name, version
            )
        } else {
            format!("{}:{}/{}", pkg.name.namespace, pkg.name.name, world.name)
        }
    } else {
        world.name.clone()
    }
}

#[allow(dead_code)]
pub mod fixtures {
    use super::*;

    const FIXTURE_WIT: &str = r#"
package greentic:component@0.1.0;
world node {
    export describe: func();
}
"#;

    pub fn good_component() -> TestComponent {
        TestComponent::new(FIXTURE_WIT, &["describe"])
    }

    pub fn bad_world_component() -> TestComponent {
        let mut component = good_component();
        let mut value: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&component.manifest_path).expect("manifest read"),
        )
        .expect("manifest json");
        value["world"] = serde_json::Value::String("greentic:component/bad@0.1.0".into());
        fs::write(
            &component.manifest_path,
            serde_json::to_string_pretty(&value).unwrap(),
        )
        .expect("rewrite manifest");
        component.manifest = parse_manifest(&value.to_string()).expect("parse mutated manifest");
        component
    }
}

fn detect_world(bytes: &[u8]) -> Option<String> {
    let decoded = greentic_component::wasm::decode_world(bytes).ok()?;
    Some(world_label(&decoded.resolve, decoded.world))
}
