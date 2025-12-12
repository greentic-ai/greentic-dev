use std::fs;
use std::path::Path;

use greentic_component::manifest::parse_manifest;
use greentic_component::telemetry::span_name;

#[test]
fn span_name_uses_manifest_prefix_when_available() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/manifests/valid.component.json");
    let raw = fs::read_to_string(path).unwrap();
    let manifest = parse_manifest(&raw).unwrap();
    let name = span_name(&manifest, "invoke");
    assert_eq!(name, "component.echo/invoke");
}

#[test]
fn span_name_falls_back_to_id() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/manifests/valid.component.json");
    let mut manifest = parse_manifest(&fs::read_to_string(path).unwrap()).unwrap();
    manifest.telemetry = None;
    let name = span_name(&manifest, "invoke");
    assert_eq!(name, "com.greentic.demo.echo/invoke");
}
