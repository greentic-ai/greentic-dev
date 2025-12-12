//! Run-level telemetry shared between runners, CLIs, and CI integrations.

use alloc::{collections::BTreeMap, string::String, vec::Vec};

use semver::Version;

use crate::{ComponentId, FlowId, NodeId, PackId, SessionKey};

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(all(feature = "serde", feature = "time"))]
use serde_with::serde_as;
#[cfg(feature = "time")]
use time::OffsetDateTime;

/// Overall execution status emitted by the runner.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum RunStatus {
    /// Flow finished successfully.
    Success,
    /// Flow finished with partial failures but continued.
    PartialFailure,
    /// Flow failed.
    Failure,
}

/// Per-node execution status.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum NodeStatus {
    /// Node executed successfully.
    Ok,
    /// Node skipped execution (e.g. gated by conditionals).
    Skipped,
    /// Node errored.
    Error,
}

/// Aggregated timing summary per node.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct NodeSummary {
    /// Stable node identifier.
    pub node_id: NodeId,
    /// Component backing the node implementation.
    pub component: ComponentId,
    /// Final status of the node execution.
    pub status: NodeStatus,
    /// Execution time reported by the runner.
    pub duration_ms: u64,
}

/// Byte-range offsets referencing captured transcripts/logs.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct TranscriptOffset {
    /// Start offset (inclusive) counted in bytes.
    pub start: u64,
    /// End offset (exclusive) counted in bytes.
    pub end: u64,
}

/// Rich failure diagnostics for a node.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct NodeFailure {
    /// Machine readable error code.
    pub code: String,
    /// Human readable explanation.
    pub message: String,
    /// Optional structured metadata.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "BTreeMap::is_empty")
    )]
    pub details: BTreeMap<String, String>,
    /// Transcript offsets referencing the failure within captured logs.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub transcript_offsets: Vec<TranscriptOffset>,
    /// Disk paths or URIs pointing at log bundles.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub log_paths: Vec<String>,
}

/// Aggregated run outcome emitted by the runtime.
#[cfg(feature = "time")]
#[cfg_attr(feature = "serde", serde_as)]
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct RunResult {
    /// Session identifier emitted by the runtime.
    pub session_id: SessionKey,
    /// Pack identifier.
    pub pack_id: PackId,
    /// Pack version executed for the run.
    #[cfg_attr(
        feature = "serde",
        serde_as(as = "serde_with::formats::DisplayFromStr")
    )]
    #[cfg_attr(
        feature = "schemars",
        schemars(with = "String", description = "SemVer version")
    )]
    pub pack_version: Version,
    /// Flow identifier executed for the session.
    pub flow_id: FlowId,
    /// Wall-clock start timestamp in UTC.
    #[cfg_attr(
        feature = "schemars",
        schemars(with = "String", description = "RFC3339 timestamp (UTC)")
    )]
    pub started_at_utc: OffsetDateTime,
    /// Wall-clock finish timestamp in UTC.
    #[cfg_attr(
        feature = "schemars",
        schemars(with = "String", description = "RFC3339 timestamp (UTC)")
    )]
    pub finished_at_utc: OffsetDateTime,
    /// Final run status.
    pub status: RunStatus,
    /// Per-node execution summaries.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub node_summaries: Vec<NodeSummary>,
    /// Rich failure diagnostics, if any.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub failures: Vec<NodeFailure>,
    /// Directory containing emitted artifacts/log bundles.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub artifacts_dir: Option<String>,
}

#[cfg(feature = "time")]
impl RunResult {
    /// Returns the total duration in milliseconds.
    pub fn duration_ms(&self) -> u64 {
        let duration = self.finished_at_utc - self.started_at_utc;
        duration.whole_milliseconds().max(0) as u64
    }
}
