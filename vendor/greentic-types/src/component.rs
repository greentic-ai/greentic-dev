//! Component manifest structures with generic capability declarations.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use semver::Version;

use crate::flow::FlowKind;
use crate::{ComponentId, FlowId};

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Component metadata describing capabilities and supported flows.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct ComponentManifest {
    /// Logical component identifier (opaque string).
    pub id: ComponentId,
    /// Semantic component version.
    #[cfg_attr(
        feature = "schemars",
        schemars(with = "String", description = "SemVer version")
    )]
    pub version: Version,
    /// Flow kinds this component can participate in.
    #[cfg_attr(feature = "serde", serde(default))]
    pub supports: Vec<FlowKind>,
    /// Referenced WIT world binding.
    pub world: String,
    /// Profile metadata for the component.
    pub profiles: ComponentProfiles,
    /// Capability contract required by the component.
    pub capabilities: ComponentCapabilities,
    /// Optional configurator flows.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub configurators: Option<ComponentConfigurators>,
    /// Operation-level descriptions.
    #[cfg_attr(feature = "serde", serde(default))]
    pub operations: Vec<ComponentOperation>,
    /// Optional configuration schema.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub config_schema: Option<serde_json::Value>,
    /// Resource usage hints for deployers/schedulers.
    #[cfg_attr(feature = "serde", serde(default))]
    pub resources: ResourceHints,
    /// Optional development-time flows attached to this component.
    ///
    /// These are used by authoring tools and may be ignored by runtimes.
    #[cfg_attr(feature = "serde", serde(default))]
    pub dev_flows: BTreeMap<FlowId, ComponentDevFlow>,
}

impl ComponentManifest {
    /// Returns `true` when the component supports the specified flow kind.
    pub fn supports_kind(&self, kind: FlowKind) -> bool {
        self.supports.iter().copied().any(|entry| entry == kind)
    }

    /// Resolves the effective profile name, returning the requested profile when supported or
    /// falling back to the manifest default.
    pub fn select_profile<'a>(
        &'a self,
        requested: Option<&str>,
    ) -> Result<Option<&'a str>, ComponentProfileError> {
        if let Some(name) = requested {
            let matched = self
                .profiles
                .supported
                .iter()
                .find(|candidate| candidate.as_str() == name)
                .ok_or_else(|| ComponentProfileError::UnsupportedProfile {
                    requested: name.to_owned(),
                    supported: self.profiles.supported.clone(),
                })?;
            Ok(Some(matched.as_str()))
        } else {
            Ok(self.profiles.default.as_deref())
        }
    }

    /// Returns the optional basic configurator flow identifier.
    pub fn basic_configurator(&self) -> Option<&FlowId> {
        self.configurators
            .as_ref()
            .and_then(|cfg| cfg.basic.as_ref())
    }

    /// Returns the optional full configurator flow identifier.
    pub fn full_configurator(&self) -> Option<&FlowId> {
        self.configurators
            .as_ref()
            .and_then(|cfg| cfg.full.as_ref())
    }
}

/// Development-time flow attached to a component manifest.
///
/// This is intended for authoring tools (`greentic-dev`, `greentic-component`)
/// and is not required by runtimes.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct ComponentDevFlow {
    /// Flow representation format. Currently only `flow-ir-json` is supported.
    #[cfg_attr(feature = "serde", serde(default = "dev_flow_default_format"))]
    pub format: String,
    /// FlowIR JSON graph for this flow.
    pub graph: serde_json::Value,
}

fn dev_flow_default_format() -> String {
    "flow-ir-json".to_owned()
}

/// Component profile declaration.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct ComponentProfiles {
    /// Default profile applied when a node does not specify one.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub default: Option<String>,
    /// Supported profile identifiers.
    #[cfg_attr(feature = "serde", serde(default))]
    pub supported: Vec<String>,
}

/// Flow configurators linked from a component manifest.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct ComponentConfigurators {
    /// Basic configurator flow identifier.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub basic: Option<FlowId>,
    /// Full configurator flow identifier.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub full: Option<FlowId>,
}

/// Operation descriptor for a component.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct ComponentOperation {
    /// Operation name (for example `handle_message`).
    pub name: String,
    /// Input schema for the operation.
    pub input_schema: serde_json::Value,
    /// Output schema for the operation.
    pub output_schema: serde_json::Value,
}

/// Resource usage hints for a component.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct ResourceHints {
    /// Suggested CPU in millis.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub cpu_millis: Option<u32>,
    /// Suggested memory in MiB.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub memory_mb: Option<u32>,
    /// Expected average latency in milliseconds.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub average_latency_ms: Option<u32>,
}

/// Host + WASI capabilities required by a component.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct ComponentCapabilities {
    /// WASI Preview 2 surfaces.
    pub wasi: WasiCapabilities,
    /// Host capability surfaces.
    pub host: HostCapabilities,
}

/// WASI capability declarations.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct WasiCapabilities {
    /// Filesystem configuration.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub filesystem: Option<FilesystemCapabilities>,
    /// Environment variable allow list.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub env: Option<EnvCapabilities>,
    /// Whether random number generation is required.
    #[cfg_attr(feature = "serde", serde(default))]
    pub random: bool,
    /// Whether clock access is required.
    #[cfg_attr(feature = "serde", serde(default))]
    pub clocks: bool,
}

/// Filesystem sandbox configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct FilesystemCapabilities {
    /// Filesystem exposure mode.
    pub mode: FilesystemMode,
    /// Declared mounts.
    #[cfg_attr(feature = "serde", serde(default))]
    pub mounts: Vec<FilesystemMount>,
}

impl Default for FilesystemCapabilities {
    fn default() -> Self {
        Self {
            mode: FilesystemMode::None,
            mounts: Vec::new(),
        }
    }
}

/// Filesystem exposure mode.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum FilesystemMode {
    /// No filesystem access.
    #[default]
    None,
    /// Read-only view with predefined mounts.
    ReadOnly,
    /// Isolated sandbox with write access.
    Sandbox,
}

/// Single mount definition.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct FilesystemMount {
    /// Logical mount identifier.
    pub name: String,
    /// Host-provided storage class (scratch/cache/config/etc.).
    pub host_class: String,
    /// Guest-visible mount path.
    pub guest_path: String,
}

/// Environment variable allow list.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct EnvCapabilities {
    /// Environment variable names components may read.
    #[cfg_attr(feature = "serde", serde(default))]
    pub allow: Vec<String>,
}

/// Host capability declaration.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct HostCapabilities {
    /// Secret resolution requirements.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub secrets: Option<SecretsCapabilities>,
    /// Durable state access requirements.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub state: Option<StateCapabilities>,
    /// Messaging ingress/egress needs.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub messaging: Option<MessagingCapabilities>,
    /// Event ingress/egress needs.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub events: Option<EventsCapabilities>,
    /// HTTP client/server needs.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub http: Option<HttpCapabilities>,
    /// Telemetry emission settings.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub telemetry: Option<TelemetryCapabilities>,
    /// Infrastructure-as-code artifact permissions.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub iac: Option<IaCCapabilities>,
}

/// Secret requirements.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct SecretsCapabilities {
    /// Secret identifiers required at runtime.
    #[cfg_attr(feature = "serde", serde(default))]
    pub required: Vec<String>,
}

/// State surface declaration.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct StateCapabilities {
    /// Whether read access is required.
    #[cfg_attr(feature = "serde", serde(default))]
    pub read: bool,
    /// Whether write access is required.
    #[cfg_attr(feature = "serde", serde(default))]
    pub write: bool,
}

/// Messaging capability declaration.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct MessagingCapabilities {
    /// Whether the component receives inbound messages.
    #[cfg_attr(feature = "serde", serde(default))]
    pub inbound: bool,
    /// Whether the component emits outbound messages.
    #[cfg_attr(feature = "serde", serde(default))]
    pub outbound: bool,
}

/// Events capability declaration.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct EventsCapabilities {
    /// Whether inbound events are handled.
    #[cfg_attr(feature = "serde", serde(default))]
    pub inbound: bool,
    /// Whether outbound events are emitted.
    #[cfg_attr(feature = "serde", serde(default))]
    pub outbound: bool,
}

/// HTTP capability declaration.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct HttpCapabilities {
    /// Outbound HTTP client usage.
    #[cfg_attr(feature = "serde", serde(default))]
    pub client: bool,
    /// Inbound HTTP server usage.
    #[cfg_attr(feature = "serde", serde(default))]
    pub server: bool,
}

/// Telemetry scoping modes.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum TelemetryScope {
    /// Emitted telemetry is scoped to the tenant.
    Tenant,
    /// Scoped to the pack.
    Pack,
    /// Scoped per-node invocation.
    Node,
}

/// Telemetry capability declaration.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct TelemetryCapabilities {
    /// Maximum telemetry scope granted to the component.
    pub scope: TelemetryScope,
}

/// Infrastructure-as-code host permissions.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct IaCCapabilities {
    /// Whether templates/manifests may be written to a preopened path.
    pub write_templates: bool,
    /// Whether the component may trigger IaC plan execution via the host.
    #[cfg_attr(feature = "serde", serde(default))]
    pub execute_plans: bool,
}

/// Profile resolution errors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComponentProfileError {
    /// Requested profile is not advertised by the component.
    UnsupportedProfile {
        /// Profile requested by the flow.
        requested: String,
        /// Known supported profiles for troubleshooting.
        supported: Vec<String>,
    },
}

impl core::fmt::Display for ComponentProfileError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ComponentProfileError::UnsupportedProfile {
                requested,
                supported,
            } => {
                write!(
                    f,
                    "profile `{requested}` is not supported; known profiles: {supported:?}"
                )
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ComponentProfileError {}
