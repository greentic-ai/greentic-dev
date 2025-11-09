use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::TOOL_NAME;

#[derive(Debug, Deserialize, Serialize)]
pub struct ComponentTemplatesResponse {
    pub templates: Vec<ComponentTemplate>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ComponentTemplate {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ComponentNewResponse {
    pub scaffold: Value,
    #[serde(default)]
    pub compile_check: Option<Value>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl ComponentNewResponse {
    pub fn into_wrapper(self) -> Value {
        let mut wrapper = serde_json::json!({
            "tool": TOOL_NAME,
            "command": "new",
            "ok": true,
            "scaffold": self.scaffold,
        });
        if let Some(compile_check) = self.compile_check {
            wrapper
                .as_object_mut()
                .expect("wrapper is object")
                .insert("compile_check".to_string(), compile_check);
        }
        if !self.extra.is_empty() {
            wrapper
                .as_object_mut()
                .expect("wrapper is object")
                .insert("payload".to_string(), Value::Object(self.extra));
        }
        wrapper
    }
}
