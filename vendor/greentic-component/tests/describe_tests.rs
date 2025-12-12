#![cfg(feature = "prepare")]

#[path = "support/mod.rs"]
mod support;

use greentic_component::describe::{from_wit_world, load as load_describe};
use serde_json::json;
use support::{TestComponent, write_embedded_payload};

const TEST_WIT: &str = r#"
package greentic:component@0.1.0;
world node {
    export describe: func();
    export init: func();
}
"#;

#[test]
fn builds_payload_from_metadata() {
    let component = TestComponent::new(TEST_WIT, &["describe", "init"]);
    let payload = from_wit_world(&component.wasm_path, &component.world).unwrap();
    assert!(payload.schema_id.is_some());
    assert_eq!(
        payload.versions[0].schema["functions"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn falls_back_to_embedded_payload() {
    let component = TestComponent::new(TEST_WIT, &["describe"]);

    // Overwrite wasm so metadata decode fails and embedded payload is used
    std::fs::write(&component.wasm_path, b"wasm32-wasip2").unwrap();
    let payload_json = json!({
        "name": "embedded",
        "versions": [
            {
                "version": "0.1.0",
                "schema": {"world": "embedded"}
            }
        ]
    });
    write_embedded_payload(component.dir.path(), &payload_json);

    let payload = load_describe(&component.wasm_path, &component.manifest).unwrap();
    assert_eq!(payload.name, "embedded");
}
