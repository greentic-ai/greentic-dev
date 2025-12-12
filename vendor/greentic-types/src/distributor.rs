//! Distributor API data transfer objects used by runner/deployer clients.
//!
//! These mirror the `greentic:distributor-api@1.0.0` WIT shapes.

use alloc::string::String;

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::TenantCtx;

/// Identifier for a distributor environment.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct DistributorEnvironmentId(pub String);

impl DistributorEnvironmentId {
    /// Returns the underlying identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for DistributorEnvironmentId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for DistributorEnvironmentId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

/// Digest for a component artifact.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct ComponentDigest(pub String);

impl ComponentDigest {
    /// Returns the digest as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Heuristic check for sha256-like digests: `sha256:` + 64 lowercase hex chars.
    pub fn is_sha256_like(&self) -> bool {
        const PREFIX: &str = "sha256:";
        let s = self.0.as_str();
        if !s.starts_with(PREFIX) {
            return false;
        }
        let rest = &s[PREFIX.len()..];
        if rest.len() != 64 {
            return false;
        }
        rest.chars()
            .all(|c| c.is_ascii_hexdigit() && c.is_ascii_lowercase() || c.is_ascii_digit())
    }
}

impl From<String> for ComponentDigest {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for ComponentDigest {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

/// Resolution status for a component.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum ComponentStatus {
    /// Resolution in progress or awaiting cache.
    Pending,
    /// Component is ready for use.
    Ready,
    /// Resolution failed with a reason.
    Failed {
        /// Human-readable failure explanation.
        reason: String,
    },
}

/// Location of the resolved artifact.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(tag = "kind", rename_all = "snake_case"))]
pub enum ArtifactLocation {
    /// Local file path on disk.
    FilePath {
        /// Absolute or relative path to the artifact.
        path: String,
    },
    /// OCI reference to the artifact.
    OciReference {
        /// Reference string to the OCI artifact.
        reference: String,
    },
    /// Internal distributor handle.
    DistributorInternal {
        /// Opaque handle understood by the distributor.
        handle: String,
    },
}

/// Summary of artifact signature verification.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct SignatureSummary {
    /// Whether the signature verified.
    pub verified: bool,
    /// Signer identifier or key hint.
    pub signer: String,
    /// Opaque extra details.
    pub extra: Value,
}

/// Cache metadata for resolved artifacts.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct CacheInfo {
    /// Size of the cached artifact in bytes.
    pub size_bytes: u64,
    /// Last use timestamp in ISO 8601 (UTC).
    pub last_used_utc: String,
    /// Last refresh timestamp in ISO 8601 (UTC).
    pub last_refreshed_utc: String,
}

/// Request to resolve a component for a tenant/environment.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct ResolveComponentRequest {
    /// Tenant context for the request.
    pub tenant: TenantCtx,
    /// Distributor environment identifier.
    pub environment_id: DistributorEnvironmentId,
    /// Pack identifier.
    pub pack_id: String,
    /// Component identifier.
    pub component_id: String,
    /// Requested version or label.
    pub version: String,
    /// Opaque extension field.
    pub extra: Value,
}

/// Response returned by the distributor.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct ResolveComponentResponse {
    /// Resolution status.
    pub status: ComponentStatus,
    /// Content digest of the component.
    pub digest: ComponentDigest,
    /// Location of the resolved artifact.
    pub artifact: ArtifactLocation,
    /// Signature summary.
    pub signature: SignatureSummary,
    /// Cache metadata.
    pub cache: CacheInfo,
}
