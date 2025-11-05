use serde_json::json;
use serde_json::Value;

pub fn describe() -> Value {
    let node_schema: Value = serde_json::from_str(include_str!(
        "../schemas/v1/{{component_kebab}}.node.schema.json"
    ))
    .expect("valid node schema");

    json!({
        "component": "{{component_name}}",
        "version": 1,
        "schemas": {
            "node": node_schema
        }
    })
}
