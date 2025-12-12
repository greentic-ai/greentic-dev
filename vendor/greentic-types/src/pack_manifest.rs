//! Canonical pack manifest (.gtpack) representation embedding flows and components.

use alloc::string::String;
use alloc::vec::Vec;

use semver::Version;

use crate::{ComponentManifest, Flow, FlowId, FlowKind, PackId, SemverReq, Signature};

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Hint describing the primary purpose of a pack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum PackKind {
    /// Application packs.
    Application,
    /// Provider packs exporting components.
    Provider,
    /// Infrastructure packs.
    Infrastructure,
    /// Library packs.
    Library,
}

/// Pack manifest describing bundled flows and components.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "schemars",
    derive(JsonSchema),
    schemars(
        title = "Greentic PackManifest v1",
        description = "Canonical pack manifest embedding flows, components, dependencies and signatures.",
        rename = "greentic.pack-manifest.v1"
    )
)]
pub struct PackManifest {
    /// Schema version for the pack manifest.
    pub schema_version: String,
    /// Logical pack identifier.
    pub pack_id: PackId,
    /// Pack semantic version.
    #[cfg_attr(
        feature = "schemars",
        schemars(with = "String", description = "SemVer version")
    )]
    pub version: Version,
    /// Pack kind hint.
    pub kind: PackKind,
    /// Pack publisher.
    pub publisher: String,
    /// Component descriptors bundled within the pack.
    #[cfg_attr(feature = "serde", serde(default))]
    pub components: Vec<ComponentManifest>,
    /// Flow entries embedded in the pack.
    #[cfg_attr(feature = "serde", serde(default))]
    pub flows: Vec<PackFlowEntry>,
    /// Pack dependencies.
    #[cfg_attr(feature = "serde", serde(default))]
    pub dependencies: Vec<PackDependency>,
    /// Capability declarations for the pack.
    #[cfg_attr(feature = "serde", serde(default))]
    pub capabilities: Vec<ComponentCapability>,
    /// Pack signatures.
    #[cfg_attr(feature = "serde", serde(default))]
    pub signatures: PackSignatures,
}

/// Flow entry embedded in a pack.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct PackFlowEntry {
    /// Flow identifier.
    pub id: FlowId,
    /// Flow kind.
    pub kind: FlowKind,
    /// Flow definition.
    pub flow: Flow,
    /// Flow tags.
    #[cfg_attr(feature = "serde", serde(default))]
    pub tags: Vec<String>,
    /// Additional entrypoint identifiers for discoverability.
    #[cfg_attr(feature = "serde", serde(default))]
    pub entrypoints: Vec<String>,
}

/// Dependency entry referencing another pack.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct PackDependency {
    /// Local alias for the dependency.
    pub alias: String,
    /// Referenced pack identifier.
    pub pack_id: PackId,
    /// Required version.
    pub version_req: SemverReq,
    /// Required capabilities.
    #[cfg_attr(feature = "serde", serde(default))]
    pub required_capabilities: Vec<String>,
}

/// Named capability advertised by a pack or component collection.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct ComponentCapability {
    /// Capability name.
    pub name: String,
    /// Optional description or metadata.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub description: Option<String>,
}

/// Signature bundle accompanying a pack manifest.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct PackSignatures {
    /// Optional detached signatures.
    #[cfg_attr(feature = "serde", serde(default))]
    pub signatures: Vec<Signature>,
}
