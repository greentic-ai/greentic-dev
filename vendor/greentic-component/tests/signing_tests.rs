#![cfg(feature = "prepare")]

#[path = "support/mod.rs"]
mod support;

use greentic_component::signing::{SigningError, verify_manifest_hash, verify_wasm_hash};
use support::TestComponent;

const TEST_WIT: &str = r#"
package greentic:component@0.1.0;
world node {
    export describe: func();
}
"#;

#[test]
fn verifies_hash_successfully() {
    let component = TestComponent::new(TEST_WIT, &["describe"]);
    verify_manifest_hash(&component.manifest, component.dir.path()).unwrap();
}

#[test]
fn detects_hash_mismatch() {
    let component = TestComponent::new(TEST_WIT, &["describe"]);
    std::fs::write(&component.wasm_path, b"corrupt-wasm32-wasip2").unwrap();
    let err = verify_wasm_hash(
        component.manifest.hashes.component_wasm.as_str(),
        &component.wasm_path,
    )
    .unwrap_err();
    matches!(err, SigningError::HashMismatch { .. })
        .then_some(())
        .unwrap();
}
