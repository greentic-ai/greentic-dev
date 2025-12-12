#![allow(dead_code)]

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf()
}

fn workspace_replacements() -> &'static [String] {
    static REPLACEMENTS: OnceLock<Vec<String>> = OnceLock::new();
    REPLACEMENTS.get_or_init(|| {
        let root = workspace_root();
        let raw = root.to_string_lossy().into_owned();
        let mut variants = vec![raw.clone()];
        let slash_variant = raw.replace('\\', "/");
        if slash_variant != raw {
            variants.push(slash_variant);
        }
        variants
    })
}

pub fn normalize_text(input: &str) -> String {
    workspace_replacements()
        .iter()
        .fold(input.to_string(), |acc, needle| {
            acc.replace(needle, "<WORKSPACE>")
        })
}

pub fn normalize_value_paths(value: &mut Value) {
    match value {
        Value::String(s) => {
            *s = normalize_text(s);
        }
        Value::Array(items) => {
            for item in items {
                normalize_value_paths(item);
            }
        }
        Value::Object(map) => {
            for val in map.values_mut() {
                normalize_value_paths(val);
            }
        }
        _ => {}
    }
}
