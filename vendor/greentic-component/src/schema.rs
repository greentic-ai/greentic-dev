use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct JsonPath(String);

impl JsonPath {
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for JsonPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Error)]
pub enum SchemaIntrospectionError {
    #[error("schema json parse failed: {0}")]
    Json(#[from] serde_json::Error),
}

pub fn collect_redactions(schema_json: &str) -> Vec<JsonPath> {
    try_collect_redactions(schema_json).expect("schema traversal failed")
}

pub fn try_collect_redactions(
    schema_json: &str,
) -> Result<Vec<JsonPath>, SchemaIntrospectionError> {
    let value: Value = serde_json::from_str(schema_json)?;
    let mut hits = Vec::new();
    walk(&value, "$", &mut |map, path| {
        if map
            .get("x-redact")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            hits.push(JsonPath::new(path));
        }
    });
    Ok(hits)
}

pub fn collect_default_annotations(
    schema_json: &str,
) -> Result<Vec<(JsonPath, String)>, SchemaIntrospectionError> {
    let value: Value = serde_json::from_str(schema_json)?;
    let mut hits = Vec::new();
    walk(&value, "$", &mut |map, path| {
        if let Some(defaulted) = map.get("x-default-applied").and_then(|v| v.as_str()) {
            hits.push((JsonPath::new(path), defaulted.to_string()));
        }
    });
    Ok(hits)
}

pub fn collect_capability_hints(
    schema_json: &str,
) -> Result<Vec<(JsonPath, String)>, SchemaIntrospectionError> {
    let value: Value = serde_json::from_str(schema_json)?;
    let mut hits = Vec::new();
    walk(&value, "$", &mut |map, path| {
        if let Some(cap) = map.get("x-capability").and_then(|v| v.as_str()) {
            hits.push((JsonPath::new(path), cap.to_string()));
        }
    });
    Ok(hits)
}

fn walk(
    value: &Value,
    path: &str,
    visitor: &mut dyn FnMut(&serde_json::Map<String, Value>, String),
) {
    if let Value::Object(map) = value {
        visitor(map, path.to_string());

        if let Some(Value::Object(props)) = map.get("properties") {
            for (key, child) in props {
                let child_path = push(path, key);
                walk(child, &child_path, visitor);
            }
        }

        if let Some(Value::Object(pattern_props)) = map.get("patternProperties") {
            for (key, child) in pattern_props {
                let next = format!("{path}.patternProperties[{key}]");
                walk(child, &next, visitor);
            }
        }

        if let Some(items) = map.get("items") {
            let next = format!("{path}[*]");
            walk(items, &next, visitor);
        }

        if let Some(Value::Array(all_of)) = map.get("allOf") {
            for (idx, child) in all_of.iter().enumerate() {
                let next = format!("{path}.allOf[{idx}]");
                walk(child, &next, visitor);
            }
        }

        if let Some(Value::Array(any_of)) = map.get("anyOf") {
            for (idx, child) in any_of.iter().enumerate() {
                let next = format!("{path}.anyOf[{idx}]");
                walk(child, &next, visitor);
            }
        }

        if let Some(Value::Array(one_of)) = map.get("oneOf") {
            for (idx, child) in one_of.iter().enumerate() {
                let next = format!("{path}.oneOf[{idx}]");
                walk(child, &next, visitor);
            }
        }
    }
}

fn push(base: &str, segment: &str) -> String {
    if base == "$" {
        format!("$.{segment}")
    } else {
        format!("{base}.{segment}")
    }
}
