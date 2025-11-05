use jsonschema::{Draft, JSONSchema};
use serde_json::Value;
use serde_yaml_bw::Value as YamlValue;

pub fn validate_yaml_against_schema(yaml: &YamlValue, schema_json: &str) -> Result<(), String> {
    let json = serde_json::to_value(yaml)
        .map_err(|error| format!("could not convert YAML to JSON: {error}"))?;
    let schema_value: Value = serde_json::from_str(schema_json)
        .map_err(|error| format!("invalid schema JSON: {error}"))?;

    let compiled = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&schema_value)
        .map_err(|error| format!("failed to compile schema: {error}"))?;

    if let Err(errors) = compiled.validate(&json) {
        let message = errors
            .map(|error| error.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        Err(format!("Schema validation failed: {message}"))
    } else {
        Ok(())
    }
}

pub fn schema_id_from_json(schema_json: &str) -> Option<String> {
    let value: Value = serde_json::from_str(schema_json).ok()?;
    value
        .get("$id")
        .and_then(|id| id.as_str())
        .map(|id| id.to_string())
}
