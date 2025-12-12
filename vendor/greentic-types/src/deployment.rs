//! Generic deployment planning structures shared between packs, runners, and deployers.

use alloc::string::String;
use alloc::vec::Vec;

use semver::Version;

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Provider-agnostic deployment description shared across tools.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct DeploymentPlan {
    /// Pack being deployed.
    pub pack_id: String,
    /// Pack version.
    #[cfg_attr(
        feature = "schemars",
        schemars(with = "String", description = "SemVer version")
    )]
    pub pack_version: Version,
    /// Tenant identifier.
    pub tenant: String,
    /// Environment identifier.
    pub environment: String,
    /// Logical runtime topology.
    pub runners: Vec<RunnerPlan>,
    /// Messaging fabric description.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub messaging: Option<MessagingPlan>,
    /// Channel entrypoints into the pack.
    pub channels: Vec<ChannelPlan>,
    /// Secrets required to operate the pack.
    pub secrets: Vec<SecretPlan>,
    /// OAuth client requirements.
    pub oauth: Vec<OAuthPlan>,
    /// Telemetry guidance.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub telemetry: Option<TelemetryPlan>,
    /// Free-form extension space.
    #[cfg_attr(feature = "serde", serde(default))]
    pub extra: Value,
}

/// Runner sizing and capabilities plan.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct RunnerPlan {
    /// Logical runner name.
    pub name: String,
    /// Desired concurrency level.
    pub replicas: u32,
    /// Additional hints/capabilities (opaque).
    #[cfg_attr(feature = "serde", serde(default))]
    pub capabilities: Value,
}

/// Messaging cluster plan.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct MessagingPlan {
    /// Logical cluster identifier.
    pub logical_cluster: String,
    /// Subjects/streams required by the pack.
    pub subjects: Vec<MessagingSubjectPlan>,
    /// Extension metadata.
    #[cfg_attr(feature = "serde", serde(default))]
    pub extra: Value,
}

/// Messaging subject plan entry.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct MessagingSubjectPlan {
    /// Subject/stream name.
    pub name: String,
    /// Intended use.
    pub purpose: String,
    /// Whether durability is required.
    pub durable: bool,
    /// Extension metadata.
    #[cfg_attr(feature = "serde", serde(default))]
    pub extra: Value,
}

/// Channel entrypoint description.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct ChannelPlan {
    /// Logical channel name.
    pub name: String,
    /// Flow entrypoint.
    pub flow_id: String,
    /// Connector kind (opaque).
    pub kind: String,
    /// Connector-specific configuration.
    #[cfg_attr(feature = "serde", serde(default))]
    pub config: Value,
}

/// Secret requirement entry.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct SecretPlan {
    /// Logical secret key.
    pub key: String,
    /// Whether the secret must exist.
    pub required: bool,
    /// Scope identifier (tenant/environment/etc.).
    pub scope: String,
}

/// OAuth client requirement entry.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct OAuthPlan {
    /// Provider identifier.
    pub provider_id: String,
    /// Logical client identifier.
    pub logical_client_id: String,
    /// Redirect path relative to host choice.
    pub redirect_path: String,
    /// Extension metadata.
    #[cfg_attr(feature = "serde", serde(default))]
    pub extra: Value,
}

/// Telemetry configuration hints.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct TelemetryPlan {
    /// Whether telemetry must be configured.
    pub required: bool,
    /// Optional suggested endpoint.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub suggested_endpoint: Option<String>,
    /// Extension metadata.
    #[cfg_attr(feature = "serde", serde(default))]
    pub extra: Value,
}
