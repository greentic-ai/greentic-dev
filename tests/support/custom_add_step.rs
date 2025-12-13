#![allow(dead_code)]

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result, anyhow, bail};
use serde_json::Value as JsonValue;

use super::flow::{load_default_flow, missing_default_err, validate_graph};

pub fn custom_add_step(
    manifest_path: &Path,
    component_ref: &str,
    answers: &HashMap<String, JsonValue>,
) -> Result<JsonValue> {
    let raw = std::fs::read_to_string(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let mut manifest: JsonValue = serde_json::from_str(&raw)
        .with_context(|| format!("invalid JSON in {}", manifest_path.display()))?;

    let schema = manifest
        .get("config_schema")
        .cloned()
        .unwrap_or(JsonValue::Null);
    let constraints = parse_schema(&schema)?;

    let graph = load_default_flow(manifest_path)?;
    let mut graph = graph;
    let graph_obj = graph
        .as_object_mut()
        .ok_or_else(|| anyhow!("graph must be an object"))?;
    let nodes = graph_obj
        .get_mut("nodes")
        .and_then(|v| v.as_object_mut())
        .ok_or_else(|| anyhow!("graph missing nodes"))?;

    if nodes.contains_key("custom_step") {
        bail!("custom_step already exists");
    }
    if !nodes.contains_key("start") {
        bail!("graph missing start node");
    }
    if !nodes.contains_key("end") {
        bail!("graph missing end node");
    }

    let config = build_config(&constraints, answers, component_ref)?;

    let mut new_node = serde_json::Map::new();
    new_node.insert("component.exec".to_string(), config);
    new_node.insert(
        "routing".to_string(),
        JsonValue::Array(vec![JsonValue::Object(
            [("to".to_string(), JsonValue::String("end".into()))]
                .into_iter()
                .collect(),
        )]),
    );
    nodes.insert("custom_step".to_string(), JsonValue::Object(new_node));

    // update start routing/edges
    if let Some(start) = nodes.get_mut("start").and_then(|v| v.as_object_mut()) {
        let routing = start
            .entry("routing")
            .or_insert_with(|| JsonValue::Array(Vec::new()));
        if let Some(arr) = routing.as_array_mut() {
            arr.push(JsonValue::Object(
                [("to".to_string(), JsonValue::String("custom_step".into()))]
                    .into_iter()
                    .collect(),
            ));
        }
    }
    let edges = graph_obj
        .entry("edges")
        .or_insert_with(|| JsonValue::Array(Vec::new()));
    if let Some(arr) = edges.as_array_mut() {
        arr.push(JsonValue::Object(
            [
                ("from".to_string(), JsonValue::String("start".into())),
                ("to".to_string(), JsonValue::String("custom_step".into())),
            ]
            .into_iter()
            .collect(),
        ));
        arr.push(JsonValue::Object(
            [
                ("from".to_string(), JsonValue::String("custom_step".into())),
                ("to".to_string(), JsonValue::String("end".into())),
            ]
            .into_iter()
            .collect(),
        ));
    }

    // persist graph into manifest
    if let Some(flows) = manifest
        .as_object_mut()
        .and_then(|o| o.get_mut("dev_flows"))
        .and_then(|v| v.as_object_mut())
    {
        if let Some(default) = flows.get_mut("default").and_then(|v| v.as_object_mut()) {
            default.insert("graph".to_string(), graph.clone());
        } else {
            return Err(missing_default_err());
        }
    } else {
        return Err(missing_default_err());
    }

    validate_graph(&graph)?;

    std::fs::write(manifest_path, serde_json::to_string_pretty(&manifest)?)
        .with_context(|| format!("failed to write {}", manifest_path.display()))?;

    Ok(manifest)
}

#[derive(Debug)]
struct Constraints {
    required: Vec<String>,
    ask: Vec<String>,
    types: HashMap<String, String>,
}

fn parse_schema(schema: &JsonValue) -> Result<Constraints> {
    let mut required = Vec::new();
    let mut ask = Vec::new();
    let mut types = HashMap::new();
    if let Some(req) = schema.get("required").and_then(|v| v.as_array()) {
        for entry in req {
            if let Some(name) = entry.as_str() {
                required.push(name.to_string());
            }
        }
    }
    if let Some(props) = schema.get("properties").and_then(|v| v.as_object()) {
        for (name, prop) in props {
            if let Some(t) = prop.get("type").and_then(|v| v.as_str()) {
                types.insert(name.clone(), t.to_string());
            }
            if prop
                .get("x-flow-ask")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                ask.push(name.clone());
            }
        }
    }
    Ok(Constraints {
        required,
        ask,
        types,
    })
}

fn build_config(
    constraints: &Constraints,
    answers: &HashMap<String, JsonValue>,
    component_ref: &str,
) -> Result<JsonValue> {
    let mut input = serde_json::Map::new();
    // require required fields
    for field in &constraints.required {
        let value = answers
            .get(field)
            .ok_or_else(|| anyhow!("missing required field `{field}`"))?;
        validate_type(field, value, constraints)?;
        input.insert(field.clone(), value.clone());
    }
    for field in &constraints.ask {
        if let Some(value) = answers.get(field) {
            validate_type(field, value, constraints)?;
            input.insert(field.clone(), value.clone());
        } else {
            return Err(anyhow!("missing prompted field `{field}`"));
        }
    }

    // optional provided values
    for (field, value) in answers {
        if input.contains_key(field) {
            continue;
        }
        validate_type(field, value, constraints)?;
        input.insert(field.clone(), value.clone());
    }

    Ok(JsonValue::Object(
        [
            (
                "component".to_string(),
                JsonValue::String(component_ref.to_string()),
            ),
            ("op".to_string(), JsonValue::String("configure".into())),
            ("input".to_string(), JsonValue::Object(input)),
        ]
        .into_iter()
        .collect(),
    ))
}

fn validate_type(field: &str, value: &JsonValue, constraints: &Constraints) -> Result<()> {
    if let Some(expected) = constraints.types.get(field) {
        match expected.as_str() {
            "string" if value.is_string() => Ok(()),
            "number" if value.is_number() => Ok(()),
            "boolean" if value.is_boolean() => Ok(()),
            _ => Err(anyhow!(
                "field `{field}` failed type check for `{expected}`"
            )),
        }
    } else {
        Ok(())
    }
}
