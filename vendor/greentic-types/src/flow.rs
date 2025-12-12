//! Unified flow model used by packs and runtimes.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use core::hash::BuildHasherDefault;

use fnv::FnvHasher;
use indexmap::IndexMap;
use serde_json::Value;

use crate::{ComponentId, FlowId, NodeId};

/// Build hasher used for flow node maps (Fnv for `no_std` friendliness).
pub type FlowHasher = BuildHasherDefault<FnvHasher>;

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Supported flow kinds across Greentic packs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum FlowKind {
    /// Inbound messaging flows (Telegram, Teams, HTTP chat).
    Messaging,
    /// Event-driven flows (webhooks, NATS, cron, etc.).
    Event,
    /// Flows that configure components/providers/infrastructure.
    ComponentConfig,
    /// Batch/background jobs.
    Job,
    /// HTTP-style request/response flows.
    Http,
}

/// Canonical flow representation embedded in packs.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "schemars",
    derive(JsonSchema),
    schemars(
        title = "Greentic Flow v1",
        description = "Canonical flow model with components, routing and telemetry.",
        rename = "greentic.flow.v1"
    )
)]
pub struct Flow {
    /// Schema version for the flow document.
    pub schema_version: String,
    /// Flow identifier inside the pack.
    pub id: FlowId,
    /// Flow execution kind.
    pub kind: FlowKind,
    /// Entrypoints for this flow keyed by name (for example `default`, `telegram`, `http:/path`).
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(
        feature = "schemars",
        schemars(with = "alloc::collections::BTreeMap<String, Value>")
    )]
    pub entrypoints: BTreeMap<String, Value>,
    /// Ordered node map describing the flow graph.
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(
        feature = "schemars",
        schemars(with = "alloc::collections::BTreeMap<NodeId, Node>")
    )]
    pub nodes: IndexMap<NodeId, Node, FlowHasher>,
    /// Optional metadata for authoring/UX.
    #[cfg_attr(feature = "serde", serde(default))]
    pub metadata: FlowMetadata,
}

impl Flow {
    /// Returns `true` when no nodes are defined.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns the implicit ingress node (first user-declared entry).
    pub fn ingress(&self) -> Option<(&NodeId, &Node)> {
        self.nodes.iter().next()
    }
}

/// Flow node representation.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct Node {
    /// Node identifier.
    pub id: NodeId,
    /// Component binding referenced by the node.
    pub component: ComponentRef,
    /// Component input mapping configuration.
    pub input: InputMapping,
    /// Component output mapping configuration.
    pub output: OutputMapping,
    /// Routing behaviour after this node.
    pub routing: Routing,
    /// Optional telemetry hints for this node.
    #[cfg_attr(feature = "serde", serde(default))]
    pub telemetry: TelemetryHints,
}

/// Component reference within a flow.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct ComponentRef {
    /// Component identifier.
    pub id: ComponentId,
    /// Dependency pack alias when referencing external components.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub pack_alias: Option<String>,
    /// Optional operation name within the component.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub operation: Option<String>,
}

/// Opaque component input mapping configuration.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct InputMapping {
    /// Mapping configuration (templates, expressions, etc.).
    #[cfg_attr(feature = "serde", serde(default))]
    pub mapping: Value,
}

/// Opaque component output mapping configuration.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct OutputMapping {
    /// Mapping configuration (templates, expressions, etc.).
    #[cfg_attr(feature = "serde", serde(default))]
    pub mapping: Value,
}

/// Optional authoring metadata for flows.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct FlowMetadata {
    /// Optional human-friendly title.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub title: Option<String>,
    /// Optional human-friendly description.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub description: Option<String>,
    /// Optional tags.
    #[cfg_attr(feature = "serde", serde(default))]
    pub tags: BTreeSet<String>,
    /// Free-form metadata.
    #[cfg_attr(feature = "serde", serde(default))]
    pub extra: Value,
}

impl Default for FlowMetadata {
    fn default() -> Self {
        Self {
            title: None,
            description: None,
            tags: BTreeSet::new(),
            extra: Value::Null,
        }
    }
}

/// Routing behaviour for a node.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum Routing {
    /// Continue to the specified node.
    Next {
        /// Destination node identifier.
        node_id: NodeId,
    },
    /// Branch based on status string -> node id.
    Branch {
        /// Mapping of status value to destination node.
        #[cfg_attr(feature = "serde", serde(default))]
        #[cfg_attr(
            feature = "schemars",
            schemars(with = "alloc::collections::BTreeMap<String, NodeId>")
        )]
        on_status: BTreeMap<String, NodeId>,
        /// Default node when no status matches.
        #[cfg_attr(
            feature = "serde",
            serde(default, skip_serializing_if = "Option::is_none")
        )]
        default: Option<NodeId>,
    },
    /// Flow terminates successfully.
    End,
    /// Reply to origin (Messaging/Http flows).
    Reply,
    /// Component- or runtime-specific routing.
    Custom(Value),
}

/// Optional telemetry hints for a node.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct TelemetryHints {
    /// Optional span name.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub span_name: Option<String>,
    /// Attributes to attach to spans/logs.
    #[cfg_attr(feature = "serde", serde(default))]
    pub attributes: BTreeMap<String, String>,
    /// Sampling hint (`high`, `normal`, `low`).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub sampling: Option<String>,
}
