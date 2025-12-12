use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::manifest::ComponentManifest;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TelemetrySpec {
    pub span_prefix: String,
    #[serde(default)]
    pub attributes: BTreeMap<String, String>,
    #[serde(default = "default_emit_node_spans")]
    pub emit_node_spans: bool,
}

impl TelemetrySpec {
    pub fn attribute(&self, key: &str) -> Option<&String> {
        self.attributes.get(key)
    }
}

fn default_emit_node_spans() -> bool {
    true
}

pub fn span_name(component: &ComponentManifest, operation: &str) -> String {
    let prefix = component
        .telemetry
        .as_ref()
        .map(|spec| spec.span_prefix.as_str())
        .unwrap_or_else(|| component.id.as_str());
    format!("{prefix}/{operation}")
}
