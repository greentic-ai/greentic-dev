use std::borrow::Cow;
use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use greentic_component::wasm;
use wasm_encoder::{
    CodeSection, CustomSection, ExportKind, ExportSection, Function, FunctionSection, Instruction,
    Module, TypeSection,
};
use wit_component::{StringEncoding, metadata};
use wit_parser::{Resolve, WorldId};

const WASI_MARKER: &str = "wasm32-wasip2";
const FIXTURE_WIT: &str = r#"
package greentic:component;

world echo {
    export describe: func();
}
"#;

fn main() -> Result<()> {
    let (bytes, world_label) = build_module(FIXTURE_WIT, &["describe"]);
    let dir = PathBuf::from("crates/greentic-component/tests/fixtures/manifests/bin");
    fs::create_dir_all(&dir)?;
    let wasm_path = dir.join("component.wasm");
    fs::write(&wasm_path, &bytes)?;
    let hash = format!("blake3:{}", hex::encode(blake3::hash(&bytes).as_bytes()));
    println!("fixture world label: {world_label}");
    println!("component_wasm hash: {hash}");
    Ok(())
}

fn build_module(world_src: &str, funcs: &[&str]) -> (Vec<u8>, String) {
    let mut resolve = Resolve::default();
    let pkg = resolve
        .push_str("fixture.wit", world_src)
        .expect("push wit");
    let world = resolve
        .select_world(&[pkg], Some("echo"))
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

fn detect_world(bytes: &[u8]) -> Option<String> {
    let decoded = wasm::decode_world(bytes).ok()?;
    Some(world_label(&decoded.resolve, decoded.world))
}
