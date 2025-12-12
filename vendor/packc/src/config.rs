use crate::path_safety::normalize_under_root;
use anyhow::{Context, Result};
use greentic_types::{ComponentCapabilities, ComponentProfiles, FlowKind, ResourceHints};
use serde::Deserialize;
use serde_json::Value as JsonValue;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct PackConfig {
    pub pack_id: String,
    pub version: String,
    pub kind: String,
    pub publisher: String,
    #[serde(default)]
    pub components: Vec<ComponentConfig>,
    #[serde(default)]
    pub dependencies: Vec<DependencyConfig>,
    #[serde(default)]
    pub flows: Vec<FlowConfig>,
    #[serde(default)]
    pub assets: Vec<AssetConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ComponentConfig {
    pub id: String,
    pub version: String,
    pub world: String,
    #[serde(default)]
    pub supports: Vec<FlowKindLabel>,
    pub profiles: ComponentProfiles,
    pub capabilities: ComponentCapabilities,
    pub wasm: PathBuf,
    #[serde(default)]
    pub operations: Vec<ComponentOperationConfig>,
    #[serde(default)]
    pub config_schema: Option<JsonValue>,
    #[serde(default)]
    pub resources: Option<ResourceHints>,
    #[serde(default)]
    pub configurators: Option<ComponentConfiguratorConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ComponentOperationConfig {
    pub name: String,
    pub input_schema: JsonValue,
    pub output_schema: JsonValue,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FlowConfig {
    pub id: String,
    pub file: PathBuf,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub entrypoints: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DependencyConfig {
    pub alias: String,
    pub pack_id: String,
    pub version_req: String,
    #[serde(default)]
    pub required_capabilities: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetConfig {
    pub path: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ComponentConfiguratorConfig {
    #[serde(default)]
    pub basic: Option<String>,
    #[serde(default)]
    pub full: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FlowKindLabel {
    Messaging,
    Event,
    #[serde(
        rename = "componentconfig",
        alias = "component-config",
        alias = "component_config"
    )]
    ComponentConfig,
    Job,
    Http,
}

impl FlowKindLabel {
    pub fn to_kind(&self) -> FlowKind {
        match self {
            FlowKindLabel::Messaging => FlowKind::Messaging,
            FlowKindLabel::Event => FlowKind::Event,
            FlowKindLabel::ComponentConfig => FlowKind::ComponentConfig,
            FlowKindLabel::Job => FlowKind::Job,
            FlowKindLabel::Http => FlowKind::Http,
        }
    }
}

pub fn load_pack_config(root: &Path) -> Result<PackConfig> {
    let manifest_path = normalize_under_root(root, Path::new("pack.yaml"))?;
    let contents = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let mut cfg: PackConfig = serde_yaml_bw::from_str(&contents)
        .with_context(|| format!("{} is not a valid pack.yaml", manifest_path.display()))?;

    // Normalize relative paths to be under the pack root so downstream logic can treat them as absolute.
    for component in cfg.components.iter_mut() {
        component.wasm = normalize_under_root(root, &component.wasm)?;
    }
    for flow in cfg.flows.iter_mut() {
        flow.file = normalize_under_root(root, &flow.file)?;
    }
    for asset in cfg.assets.iter_mut() {
        asset.path = normalize_under_root(root, &asset.path)?;
    }

    Ok(cfg)
}
