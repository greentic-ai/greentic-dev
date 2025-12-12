use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Error;
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use thiserror::Error;
use wit_parser::{Resolve, WorldId, WorldItem, WorldKey};

use crate::manifest::ComponentManifest;
use crate::wasm;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DescribePayload {
    pub name: String,
    pub versions: Vec<DescribeVersion>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DescribeVersion {
    pub version: Version,
    pub schema: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub defaults: Option<Value>,
}

#[derive(Debug, Error)]
pub enum DescribeError {
    #[error("failed to read describe payload at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid describe payload at {path}: {source}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to decode component metadata: {0}")]
    Metadata(Error),
    #[error("describe payload not found: {0}")]
    NotFound(String),
}

pub fn from_exported_func(
    wasm_path: &Path,
    symbol: &str,
) -> Result<DescribePayload, DescribeError> {
    let dir = wasm_path
        .parent()
        .ok_or_else(|| DescribeError::NotFound(symbol.to_string()))?;
    let candidate = dir.join(format!("{symbol}.describe.json"));
    read_payload(&candidate)
}

pub fn from_wit_world(wasm_path: &Path, _world: &str) -> Result<DescribePayload, DescribeError> {
    let bytes = fs::read(wasm_path).map_err(|source| DescribeError::Io {
        path: wasm_path.to_path_buf(),
        source,
    })?;
    let decoded = wasm::decode_world(&bytes).map_err(DescribeError::Metadata)?;
    build_payload_from_world(&decoded.resolve, decoded.world)
}

pub fn from_embedded(manifest_dir: &Path) -> Option<DescribePayload> {
    let schema_dir = manifest_dir.join("schemas").join("v1");
    let entries = fs::read_dir(schema_dir).ok()?;
    let mut files = Vec::new();
    for entry in entries.flatten() {
        files.push(entry.path());
    }
    files.sort();
    for path in files {
        if path.extension().and_then(|s| s.to_str()) == Some("json")
            && let Ok(payload) = read_payload(&path)
        {
            return Some(payload);
        }
    }
    None
}

pub fn load(
    wasm_path: &Path,
    manifest: &ComponentManifest,
) -> Result<DescribePayload, DescribeError> {
    if let Ok(payload) = from_wit_world(wasm_path, manifest.world.as_str()) {
        return Ok(payload);
    }
    if let Ok(payload) = from_exported_func(wasm_path, manifest.describe_export.as_str()) {
        return Ok(payload);
    }
    if let Some(dir) = wasm_path.parent()
        && let Some(payload) = from_embedded(dir)
    {
        return Ok(payload);
    }
    Err(DescribeError::NotFound(manifest.id.as_str().to_string()))
}

fn read_payload(path: &Path) -> Result<DescribePayload, DescribeError> {
    let data = fs::read_to_string(path).map_err(|source| DescribeError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_str(&data).map_err(|source| DescribeError::Json {
        path: path.to_path_buf(),
        source,
    })
}

fn build_payload_from_world(
    resolve: &Resolve,
    world_id: WorldId,
) -> Result<DescribePayload, DescribeError> {
    let world = &resolve.worlds[world_id];
    let world_ref = format_world(resolve, world_id);
    let version = world
        .package
        .and_then(|pkg_id| resolve.packages[pkg_id].name.version.clone())
        .map(|ver| Version::new(ver.major, ver.minor, ver.patch))
        .unwrap_or_else(|| Version::new(0, 0, 0));

    let mut functions = Vec::new();
    for (key, item) in &world.exports {
        match item {
            WorldItem::Function(func) => {
                let mut entry = Map::new();
                entry.insert("name".into(), Value::String(func.name.clone()));
                entry.insert("key".into(), Value::String(label_for_key(resolve, key)));
                if let Some(doc) = func.docs.contents.clone() {
                    entry.insert("docs".into(), Value::String(doc));
                }
                functions.push(Value::Object(entry));
            }
            WorldItem::Interface { id, .. } => {
                let iface = &resolve.interfaces[*id];
                for (name, func) in iface.functions.iter() {
                    let mut entry = Map::new();
                    entry.insert("name".into(), Value::String(name.clone()));
                    if let Some(doc) = func.docs.contents.clone() {
                        entry.insert("docs".into(), Value::String(doc));
                    }
                    if let Some(iface_name) = &iface.name {
                        entry.insert("interface".into(), Value::String(iface_name.clone()));
                    }
                    functions.push(Value::Object(entry));
                }
            }
            WorldItem::Type(_) => {}
        }
    }

    let schema = json!({
        "world": world_ref,
        "functions": functions,
    });

    Ok(DescribePayload {
        name: world.name.clone(),
        schema_id: Some(world_ref.clone()),
        versions: vec![DescribeVersion {
            version,
            schema,
            defaults: None,
        }],
    })
}

fn format_world(resolve: &Resolve, world_id: WorldId) -> String {
    let world = &resolve.worlds[world_id];
    if let Some(pkg_id) = world.package {
        let pkg = &resolve.packages[pkg_id];
        if let Some(version) = &pkg.name.version {
            format!(
                "{}:{}/{}@{}",
                pkg.name.namespace, pkg.name.name, world.name, version
            )
        } else {
            format!("{}:{}/{}", pkg.name.namespace, pkg.name.name, world.name)
        }
    } else {
        world.name.clone()
    }
}

fn label_for_key(resolve: &Resolve, key: &WorldKey) -> String {
    match key {
        WorldKey::Name(name) => name.to_string(),
        WorldKey::Interface(id) => {
            let iface = &resolve.interfaces[*id];
            iface
                .name
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("interface-{}", id.index()))
        }
    }
}
