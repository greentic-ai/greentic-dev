//! Canonical OTLP attribute keys shared across Greentic components.

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Namespaced OpenTelemetry attribute keys used by Greentic.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct OtlpKeys;

impl OtlpKeys {
    /// Service name advertised to OTLP exporters.
    pub const SERVICE_NAME: &'static str = "service.name";
    /// Pack identifier attribute.
    pub const PACK_ID: &'static str = "greentic.pack.id";
    /// Pack version attribute.
    pub const PACK_VERSION: &'static str = "greentic.pack.version";
    /// Flow identifier attribute.
    pub const FLOW_ID: &'static str = "greentic.flow.id";
    /// Node identifier attribute.
    pub const NODE_ID: &'static str = "greentic.node.id";
    /// Component name attribute.
    pub const COMPONENT_NAME: &'static str = "greentic.component.name";
    /// Component version attribute.
    pub const COMPONENT_VERSION: &'static str = "greentic.component.version";
    /// Tenant identifier attribute.
    pub const TENANT_ID: &'static str = "greentic.tenant.id";
    /// Team identifier attribute.
    pub const TEAM_ID: &'static str = "greentic.team.id";
    /// User identifier attribute.
    pub const USER_ID: &'static str = "greentic.user.id";
    /// Session identifier attribute.
    pub const SESSION_ID: &'static str = "greentic.session.id";
    /// Run status attribute.
    pub const RUN_STATUS: &'static str = "greentic.run.status";
    /// Capability name attribute.
    pub const CAPABILITY: &'static str = "greentic.capability";
    /// Artifact directory attribute.
    pub const ARTIFACTS_DIR: &'static str = "greentic.artifacts.dir";
}
