//! Generic worker envelope shared across runner and messaging components.

use alloc::{string::String, vec::Vec};

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::TenantCtx;

/// Request payload for invoking a worker.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct WorkerRequest {
    /// Version of the worker envelope (for example `1.0`).
    pub version: String,
    /// Tenant context propagated to the worker.
    pub tenant: TenantCtx,
    /// Identifier of the target worker (for example `greentic-repo-assistant`).
    pub worker_id: String,
    /// Optional correlation identifier for tracing requests across transports.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub correlation_id: Option<String>,
    /// Optional session identifier for conversational workers.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub session_id: Option<String>,
    /// Optional thread identifier when the worker groups messages into threads.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub thread_id: Option<String>,
    /// JSON-encoded payload forwarded to the worker; the ABI treats this as opaque.
    pub payload_json: String,
    /// UTC timestamp for when the request was created (ISO8601).
    pub timestamp_utc: String,
}

/// Individual message emitted by a worker.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct WorkerMessage {
    /// Message kind (for example `text`, `card`, `event`).
    pub kind: String,
    /// JSON-encoded message payload; workers and callers negotiate its shape.
    pub payload_json: String,
}

/// Response envelope returned by worker executions.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct WorkerResponse {
    /// Version of the worker envelope (mirrors the request).
    pub version: String,
    /// Tenant context propagated to the worker.
    pub tenant: TenantCtx,
    /// Identifier of the worker that handled the request.
    pub worker_id: String,
    /// Optional correlation identifier for tracing requests across transports.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub correlation_id: Option<String>,
    /// Optional session identifier for conversational workers.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub session_id: Option<String>,
    /// Optional thread identifier when the worker groups messages into threads.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub thread_id: Option<String>,
    /// Messages produced by the worker execution.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub messages: Vec<WorkerMessage>,
    /// UTC timestamp for when the response was produced (ISO8601).
    pub timestamp_utc: String,
}
