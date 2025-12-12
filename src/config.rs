use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct GreenticConfig {
    #[serde(default)]
    pub tools: ToolsSection,
    #[serde(default)]
    pub defaults: DefaultsSection,
    #[serde(default)]
    pub distributor: DistributorSection,
}

#[derive(Debug, Default, Deserialize)]
pub struct ToolsSection {
    #[serde(rename = "greentic-component", default)]
    pub greentic_component: ToolEntry,
    #[serde(rename = "packc", default)]
    pub packc: ToolEntry,
    #[serde(rename = "packc-path", default)]
    pub packc_path: ToolEntry,
}

#[derive(Debug, Default, Deserialize)]
pub struct ToolEntry {
    pub path: Option<PathBuf>,
}

#[allow(dead_code)]
#[derive(Debug, Default, Deserialize)]
pub struct DefaultsSection {
    #[serde(default)]
    pub component: ComponentDefaults,
}

#[allow(dead_code)]
#[derive(Debug, Default, Deserialize)]
pub struct ComponentDefaults {
    pub org: Option<String>,
    pub template: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct DistributorSection {
    /// Map of profile name -> profile configuration.
    #[serde(default, flatten)]
    pub profiles: HashMap<String, DistributorProfileConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DistributorProfileConfig {
    /// Base URL for the distributor (preferred field; falls back to `url` if set).
    #[serde(default)]
    pub base_url: Option<String>,
    /// Deprecated alias for base_url.
    #[serde(default)]
    pub url: Option<String>,
    /// API token; allow env:VAR indirection.
    #[serde(default)]
    pub token: Option<String>,
    /// Tenant identifier for distributor requests.
    #[serde(default)]
    pub tenant_id: Option<String>,
    /// Environment identifier for distributor requests.
    #[serde(default)]
    pub environment_id: Option<String>,
    /// Additional headers (optional).
    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,
}

pub fn load() -> Result<GreenticConfig> {
    let path_override = std::env::var("GREENTIC_CONFIG").ok();
    load_from(path_override.as_deref())
}

pub fn load_from(path_override: Option<&str>) -> Result<GreenticConfig> {
    let Some(path) = config_path_override(path_override) else {
        return Ok(GreenticConfig::default());
    };

    if !path.exists() {
        return Ok(GreenticConfig::default());
    }

    let raw = fs::read_to_string(&path)
        .with_context(|| format!("failed to read config at {}", path.display()))?;
    let config: GreenticConfig = toml::from_str(&raw)
        .with_context(|| format!("failed to parse config at {}", path.display()))?;
    Ok(config)
}

fn config_path_override(path_override: Option<&str>) -> Option<PathBuf> {
    if let Some(raw) = path_override {
        return Some(PathBuf::from(raw));
    }
    config_path()
}

pub fn config_path() -> Option<PathBuf> {
    // Prefer XDG-style config path, but fall back to legacy ~/.greentic/config.toml.
    if let Some(mut dir) = dirs::config_dir() {
        dir.push("greentic-dev");
        dir.push("config.toml");
        if dir.exists() {
            return Some(dir);
        }
    }
    dirs::home_dir().map(|mut home| {
        home.push(".greentic");
        home.push("config.toml");
        home
    })
}
