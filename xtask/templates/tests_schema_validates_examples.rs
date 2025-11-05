use jsonschema::{Draft, JSONSchema};

#[test]
fn example_flow_validates_against_schema() {
    let schema_json = include_str!("../schemas/v1/{{component_kebab}}.node.schema.json");
    let schema_value: serde_json::Value =
        serde_json::from_str(schema_json).expect("schema must be valid JSON");

    let compiled = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&schema_value)
        .expect("schema compiles");

    let flow_yaml = include_str!("../examples/flows/min.yaml");
    let flow_value: serde_yaml::Value =
        serde_yaml::from_str(flow_yaml).expect("example flow must parse");

    let node = flow_value
        .get("nodes")
        .and_then(|nodes| nodes.as_sequence())
        .and_then(|sequence| sequence.first())
        .expect("example flow must contain at least one node")
        .clone();

    let node_json = serde_json::to_value(node).expect("node converts to JSON");

    if let Err(errors) = compiled.validate(&node_json) {
        panic!(
            "validation failed: {}",
            errors
                .map(|error| error.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
}
