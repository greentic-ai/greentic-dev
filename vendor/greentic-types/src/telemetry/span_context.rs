//! Telemetry span context shared across providers.

use alloc::string::String;

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "time")]
use time::OffsetDateTime;

use crate::{SessionKey, TenantId};

/// Minimal telemetry context compatible with OTLP semantic conventions.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct SpanContext {
    /// Tenant identifier owning the span.
    pub tenant: TenantId,
    /// Optional session identifier emitting the span.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub session_id: Option<SessionKey>,
    /// Flow identifier.
    pub flow_id: String,
    /// Node identifier.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub node_id: Option<String>,
    /// Provider or runtime emitting the span.
    pub provider: String,
    /// Span start timestamp.
    #[cfg(feature = "time")]
    #[cfg_attr(
        feature = "serde",
        serde(
            default,
            skip_serializing_if = "Option::is_none",
            with = "time::serde::rfc3339::option"
        )
    )]
    #[cfg_attr(
        feature = "schemars",
        schemars(with = "Option<String>", description = "RFC3339 timestamp")
    )]
    pub start: Option<OffsetDateTime>,
    /// Span end timestamp.
    #[cfg(feature = "time")]
    #[cfg_attr(
        feature = "serde",
        serde(
            default,
            skip_serializing_if = "Option::is_none",
            with = "time::serde::rfc3339::option"
        )
    )]
    #[cfg_attr(
        feature = "schemars",
        schemars(with = "Option<String>", description = "RFC3339 timestamp")
    )]
    pub end: Option<OffsetDateTime>,
}

impl SpanContext {
    /// Creates a new span context with the supplied tenant and flow identifiers.
    pub fn new(tenant: TenantId, flow_id: impl Into<String>, provider: impl Into<String>) -> Self {
        Self {
            tenant,
            session_id: None,
            flow_id: flow_id.into(),
            node_id: None,
            provider: provider.into(),
            #[cfg(feature = "time")]
            start: None,
            #[cfg(feature = "time")]
            end: None,
        }
    }

    /// Sets the session identifier.
    pub fn with_session(mut self, session_id: SessionKey) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Sets the node identifier.
    pub fn with_node(mut self, node_id: impl Into<String>) -> Self {
        self.node_id = Some(node_id.into());
        self
    }

    /// Marks the span start timestamp.
    #[cfg(feature = "time")]
    pub fn started(mut self, start: OffsetDateTime) -> Self {
        self.start = Some(start);
        self
    }

    /// Marks the span end timestamp.
    #[cfg(feature = "time")]
    pub fn finished(mut self, end: OffsetDateTime) -> Self {
        self.end = Some(end);
        self
    }
}
