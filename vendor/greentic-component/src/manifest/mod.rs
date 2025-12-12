use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use jsonschema::{Validator, validator_for};
use once_cell::sync::Lazy;
use semver::Version;
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

use crate::capabilities::{
    Capabilities, ComponentConfigurators, ComponentProfiles, validate_capabilities,
};
use crate::limits::Limits;
use crate::provenance::Provenance;
use crate::telemetry::TelemetrySpec;
use greentic_types::flow::FlowKind;

static RAW_SCHEMA: &str = include_str!("../../schemas/v1/component.manifest.schema.json");

static COMPILED_SCHEMA: Lazy<Validator> = Lazy::new(|| {
    let value: Value =
        serde_json::from_str(RAW_SCHEMA).expect("component manifest schema must be valid JSON");
    validator_for(&value).expect("component manifest schema must compile")
});

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ComponentManifest {
    pub id: ManifestId,
    pub name: String,
    pub version: Version,
    #[serde(default)]
    pub supports: Vec<FlowKind>,
    pub world: World,
    #[serde(default)]
    pub capabilities: Capabilities,
    pub profiles: ComponentProfiles,
    #[serde(default)]
    pub configurators: Option<ComponentConfigurators>,
    #[serde(default)]
    pub limits: Option<Limits>,
    #[serde(default)]
    pub telemetry: Option<TelemetrySpec>,
    pub describe_export: DescribeExport,
    #[serde(default)]
    pub provenance: Option<Provenance>,
    pub artifacts: Artifacts,
    pub hashes: Hashes,
    #[serde(default)]
    pub dev_flows: BTreeMap<String, DevFlow>,
}

impl ComponentManifest {
    pub fn wasm_artifact_path(&self, root: &Path) -> PathBuf {
        root.join(&self.artifacts.component_wasm)
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct ManifestId(String);

impl ManifestId {
    fn parse(id: String) -> Result<Self, ManifestError> {
        if id.trim().is_empty() {
            return Err(ManifestError::EmptyField("id"));
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ManifestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct World(String);

impl World {
    fn parse(world: String) -> Result<Self, ManifestError> {
        if world.trim().is_empty() {
            return Err(ManifestError::InvalidWorld { world });
        }
        Ok(Self(world))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct DescribeExport(String);

impl DescribeExport {
    fn parse(export: String) -> Result<Self, ManifestError> {
        if export.trim().is_empty() {
            return Err(ManifestError::InvalidDescribeExport {
                export,
                reason: "describe_export cannot be empty".into(),
            });
        }
        Ok(Self(export))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn kind(&self) -> DescribeKind {
        if self.0.contains(':') && self.0.contains('/') {
            DescribeKind::WitWorld
        } else {
            DescribeKind::Export
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DescribeKind {
    Export,
    WitWorld,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Artifacts {
    component_wasm: PathBuf,
}

impl Artifacts {
    pub fn component_wasm(&self) -> &Path {
        &self.component_wasm
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Hashes {
    pub component_wasm: WasmHash,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct WasmHash(String);

impl WasmHash {
    fn parse(hash: String) -> Result<Self, ManifestError> {
        let Some(rest) = hash.strip_prefix("blake3:") else {
            return Err(ManifestError::InvalidHashFormat { hash });
        };
        if rest.len() != 64 || !rest.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ManifestError::InvalidHashFormat {
                hash: format!("blake3:{rest}"),
            });
        }
        Ok(Self(format!("blake3:{rest}")))
    }

    pub fn algorithm(&self) -> &str {
        "blake3"
    }

    pub fn digest(&self) -> &str {
        &self.0[7..]
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub fn schema() -> &'static str {
    RAW_SCHEMA
}

pub fn parse_manifest(raw: &str) -> Result<ComponentManifest, ManifestError> {
    let value: Value = serde_json::from_str(raw)?;
    validate_value(&value)?;
    let raw_manifest: RawManifest = serde_json::from_value(value)?;
    raw_manifest.try_into()
}

pub fn validate_manifest(raw: &str) -> Result<(), ManifestError> {
    let value: Value = serde_json::from_str(raw)?;
    validate_value(&value)
}

fn validate_value(value: &Value) -> Result<(), ManifestError> {
    let errors: Vec<String> = COMPILED_SCHEMA
        .iter_errors(value)
        .map(|err| err.to_string())
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(ManifestError::Schema(errors.join(", ")))
    }
}

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("manifest json parse failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("manifest schema validation failed: {0}")]
    Schema(String),
    #[error("world identifier is invalid: `{world}`")]
    InvalidWorld { world: String },
    #[error("manifest field `{0}` cannot be empty")]
    EmptyField(&'static str),
    #[error("component must support at least one flow kind")]
    MissingSupports,
    #[error("profiles.supported must include at least one profile identifier")]
    MissingProfiles,
    #[error("profiles.default `{default}` must be one of the supported profiles")]
    InvalidProfileDefault { default: String },
    #[error("invalid semantic version `{version}`: {source}")]
    InvalidVersion {
        version: String,
        #[source]
        source: semver::Error,
    },
    #[error("invalid describe export `{export}`: {reason}")]
    InvalidDescribeExport { export: String, reason: String },
    #[error("component wasm path must be relative (got `{path}`)")]
    InvalidArtifactPath { path: String },
    #[error("component wasm hash must be blake3:<hex> (got `{hash}`)")]
    InvalidHashFormat { hash: String },
    #[error("capability validation failed: {0}")]
    Capability(String),
    #[error("limits invalid: {0}")]
    Limits(String),
    #[error("provenance invalid: {0}")]
    Provenance(String),
}

#[derive(Debug, serde::Deserialize)]
struct RawManifest {
    id: String,
    name: String,
    version: String,
    world: String,
    #[serde(default)]
    supports: Vec<FlowKind>,
    #[serde(default)]
    capabilities: Capabilities,
    #[serde(default)]
    profiles: ComponentProfiles,
    #[serde(default)]
    configurators: Option<ComponentConfigurators>,
    #[serde(default)]
    limits: Option<Limits>,
    #[serde(default)]
    telemetry: Option<TelemetrySpec>,
    describe_export: String,
    #[serde(default)]
    provenance: Option<Provenance>,
    artifacts: RawArtifacts,
    hashes: RawHashes,
    #[serde(default)]
    dev_flows: BTreeMap<String, DevFlow>,
}

impl TryFrom<RawManifest> for ComponentManifest {
    type Error = ManifestError;

    fn try_from(raw: RawManifest) -> Result<Self, Self::Error> {
        if raw.name.trim().is_empty() {
            return Err(ManifestError::EmptyField("name"));
        }

        let id = ManifestId::parse(raw.id)?;
        let world = World::parse(raw.world)?;
        let version =
            Version::parse(&raw.version).map_err(|source| ManifestError::InvalidVersion {
                version: raw.version,
                source,
            })?;
        let describe_export = DescribeExport::parse(raw.describe_export)?;
        let artifacts = Artifacts::try_from(raw.artifacts)?;
        let hashes = Hashes::try_from(raw.hashes)?;

        if raw.supports.is_empty() {
            return Err(ManifestError::MissingSupports);
        }

        validate_profiles(&raw.profiles)?;

        if let Some(configurators) = &raw.configurators {
            validate_configurators(configurators)?;
        }

        validate_capabilities(&raw.capabilities)
            .map_err(|err| ManifestError::Capability(err.to_string()))?;

        if let Some(limits) = &raw.limits {
            limits
                .validate()
                .map_err(|err| ManifestError::Limits(err.to_string()))?;
        }

        if let Some(provenance) = &raw.provenance {
            provenance
                .validate()
                .map_err(|err| ManifestError::Provenance(err.to_string()))?;
        }

        Ok(Self {
            id,
            name: raw.name,
            version,
            world,
            supports: raw.supports,
            capabilities: raw.capabilities,
            profiles: raw.profiles,
            configurators: raw.configurators,
            limits: raw.limits,
            telemetry: raw.telemetry,
            describe_export,
            provenance: raw.provenance,
            artifacts,
            hashes,
            dev_flows: raw.dev_flows,
        })
    }
}

#[derive(Debug, serde::Deserialize)]
struct RawArtifacts {
    component_wasm: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, serde::Deserialize)]
pub struct DevFlow {
    #[serde(default = "dev_flow_default_format")]
    pub format: String,
    pub graph: serde_json::Value,
}

fn dev_flow_default_format() -> String {
    "flow-ir-json".to_string()
}

impl TryFrom<RawArtifacts> for Artifacts {
    type Error = ManifestError;

    fn try_from(value: RawArtifacts) -> Result<Self, Self::Error> {
        ensure_relative(&value.component_wasm)?;
        Ok(Artifacts {
            component_wasm: PathBuf::from(value.component_wasm),
        })
    }
}

#[derive(Debug, serde::Deserialize)]
struct RawHashes {
    component_wasm: String,
}

impl TryFrom<RawHashes> for Hashes {
    type Error = ManifestError;

    fn try_from(value: RawHashes) -> Result<Self, Self::Error> {
        Ok(Hashes {
            component_wasm: WasmHash::parse(value.component_wasm)?,
        })
    }
}

fn ensure_relative(path: &str) -> Result<(), ManifestError> {
    let path_buf = PathBuf::from(path);
    if path_buf.is_absolute() {
        return Err(ManifestError::InvalidArtifactPath {
            path: path.to_string(),
        });
    }
    if matches!(path_buf.components().next(), Some(Component::Prefix(_))) {
        return Err(ManifestError::InvalidArtifactPath {
            path: path.to_string(),
        });
    }
    Ok(())
}

fn validate_profiles(profiles: &ComponentProfiles) -> Result<(), ManifestError> {
    if profiles.supported.is_empty() {
        return Err(ManifestError::MissingProfiles);
    }
    if let Some(default) = &profiles.default
        && !profiles.supported.iter().any(|entry| entry == default)
    {
        return Err(ManifestError::InvalidProfileDefault {
            default: default.clone(),
        });
    }
    Ok(())
}

fn validate_configurators(_configurators: &ComponentConfigurators) -> Result<(), ManifestError> {
    // Flow identifiers are validated by greentic-types, so no additional checks are required.
    Ok(())
}
