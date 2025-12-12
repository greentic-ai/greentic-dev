#![cfg(feature = "prepare")]

#[path = "support/mod.rs"]
mod support;

use greentic_component::{LoadError, discover};
use support::TestComponent;

const TEST_WIT: &str = r#"
package greentic:component@0.1.0;
world node {
    export describe: func();
}
"#;

#[test]
fn discovers_from_manifest_path() {
    let component = TestComponent::new(TEST_WIT, &["describe"]);
    let handle = discover(component.manifest_path.to_str().unwrap()).unwrap();
    assert_eq!(handle.manifest.id.as_str(), "com.greentic.test.component");
    assert_eq!(handle.wasm_path, component.wasm_path);
}

#[test]
fn fails_when_hash_mismatches() {
    let component = TestComponent::new(TEST_WIT, &["describe"]);
    std::fs::write(&component.wasm_path, b"corrupt-wasm32-wasip2").unwrap();
    let err = discover(component.manifest_path.to_str().unwrap()).unwrap_err();
    matches!(err, LoadError::Signing(_)).then_some(()).unwrap();
}
