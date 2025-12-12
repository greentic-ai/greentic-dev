//! Capability and resource declarations shared between manifests and runtimes.

use alloc::{collections::BTreeMap, string::String, vec::Vec};

use crate::{AllowList, NetworkPolicy};

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Declarative capability toggles that packs may request from the runtime.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct Capabilities {
    /// Optional HTTP networking surface (maps to `http.fetch`).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub http: Option<HttpCaps>,
    /// Optional secret resolution surface (maps to `secrets.get`).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub secrets: Option<SecretsCaps>,
    /// Optional key-value store bindings.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub kv: Option<KvCaps>,
    /// Optional filesystem bindings (for embedded assets or scratch space).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub fs: Option<FsCaps>,
    /// Optional raw networking permissions.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub net: Option<NetCaps>,
    /// Optional tool invocation metadata (for MCP/tool.invoke surfaces).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub tools: Option<ToolsCaps>,
}

impl Capabilities {
    /// Creates an empty capability declaration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` when no capabilities are requested.
    pub fn is_empty(&self) -> bool {
        self.http.is_none()
            && self.secrets.is_none()
            && self.kv.is_none()
            && self.fs.is_none()
            && self.net.is_none()
            && self.tools.is_none()
    }
}

/// HTTP capability descriptor controlling outbound fetch settings.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct HttpCaps {
    /// Optional allow list applied before requests are dispatched.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub allow_list: Option<AllowList>,
    /// Maximum request/response body size in bytes (when enforced).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub max_body_bytes: Option<u64>,
}

impl HttpCaps {
    /// Creates an empty descriptor.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Secret capability descriptor enumerating runtime-provided handles.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct SecretsCaps {
    /// Secret identifiers that must be bound before execution.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub required: Vec<String>,
}

impl SecretsCaps {
    /// Creates an empty descriptor.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Key-value capability descriptor for packs that need durable storage.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct KvCaps {
    /// Allowed logical namespaces.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub namespaces: Vec<String>,
}

impl KvCaps {
    /// Creates an empty descriptor.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Filesystem bindings exposed to packs.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct FsCaps {
    /// List of host paths mapped into the pack sandbox.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub paths: Vec<String>,
    /// Whether the paths should be mounted read-only.
    #[cfg_attr(feature = "serde", serde(default = "FsCaps::default_read_only"))]
    pub read_only: bool,
}

impl Default for FsCaps {
    fn default() -> Self {
        Self {
            paths: Vec::new(),
            read_only: true,
        }
    }
}

impl FsCaps {
    const fn default_read_only() -> bool {
        true
    }

    /// Creates an empty descriptor.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Low-level networking descriptor (raw sockets, tunnels, etc.).
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct NetCaps {
    /// Network policy enforced before the runtime opens connections.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub policy: Option<NetworkPolicy>,
}

impl NetCaps {
    /// Creates an empty descriptor.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Tool invocation descriptor for packs relying on host tools.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct ToolsCaps {
    /// Tool identifiers the pack expects the host to resolve.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub allowed: Vec<String>,
}

impl ToolsCaps {
    /// Creates an empty descriptor.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Resource limit declarations respected by runtimes.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct Limits {
    /// Memory ceiling per flow instance (in megabytes).
    pub memory_mb: u32,
    /// Wall-clock budget per invocation (milliseconds).
    pub wall_time_ms: u64,
    /// Optional fuel/step counter for deterministic engines.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub fuel: Option<u64>,
    /// Optional file descriptor ceiling.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub files: Option<u32>,
}

impl Limits {
    /// Creates a new limit declaration.
    pub fn new(memory_mb: u32, wall_time_ms: u64) -> Self {
        Self {
            memory_mb,
            wall_time_ms,
            fuel: None,
            files: None,
        }
    }
}

impl Default for Limits {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

/// Telemetry publishing configuration shared by hosts and packs.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct TelemetrySpec {
    /// Prefix applied to spans emitted by the pack.
    pub span_prefix: String,
    /// Static key/value attributes added to every span/log record.
    #[cfg_attr(feature = "serde", serde(default))]
    pub attributes: BTreeMap<String, String>,
    /// Whether the runtime should emit per-node spans automatically.
    pub emit_node_spans: bool,
}

impl TelemetrySpec {
    /// Creates a telemetry specification with the provided prefix.
    pub fn new(span_prefix: impl Into<String>) -> Self {
        Self {
            span_prefix: span_prefix.into(),
            attributes: BTreeMap::new(),
            emit_node_spans: false,
        }
    }
}

impl Default for TelemetrySpec {
    fn default() -> Self {
        Self::new("greentic")
    }
}
