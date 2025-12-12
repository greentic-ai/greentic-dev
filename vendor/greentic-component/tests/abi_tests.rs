#![cfg(feature = "prepare")]

#[path = "support/mod.rs"]
mod support;

use greentic_component::abi::{AbiError, check_world, has_lifecycle};
use support::TestComponent;

const TEST_WIT: &str = r#"
package greentic:component@0.1.0;
world node {
    export describe: func();
    export init: func();
    export health: func();
    export shutdown: func();
}
"#;

#[test]
fn verifies_world_and_lifecycle() {
    let component = TestComponent::new(TEST_WIT, &["describe", "init", "health", "shutdown"]);

    if let Err(AbiError::WorldMismatch { found, .. }) =
        check_world(&component.wasm_path, &component.world)
    {
        check_world(&component.wasm_path, &found).unwrap();
    }
    let lifecycle = has_lifecycle(&component.wasm_path).unwrap();
    assert!(lifecycle.init);
    assert!(lifecycle.health);
    assert!(lifecycle.shutdown);
}

#[test]
fn rejects_mismatched_world() {
    let component = TestComponent::new(TEST_WIT, &["describe"]);

    let err = check_world(&component.wasm_path, "greentic:component/other@0.1.0").unwrap_err();
    matches!(err, AbiError::WorldMismatch { .. })
        .then_some(())
        .unwrap();
}
