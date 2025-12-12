//! Canonical Greentic event envelope shared across repos.

use alloc::{collections::BTreeMap, string::String};
use core::fmt;
use core::str::FromStr;

use chrono::{DateTime, Utc};
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{GResult, GreenticError, TenantCtx, validate_identifier};

/// Map of metadata entries propagated with an event.
pub type EventMetadata = BTreeMap<String, String>;

/// Stable identifier for an event envelope.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(try_from = "String", into = "String"))]
pub struct EventId(String);

impl EventId {
    /// Returns the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validates and constructs the identifier from the provided value.
    pub fn new(value: impl AsRef<str>) -> GResult<Self> {
        value.as_ref().parse()
    }

    /// Consumes the identifier returning the owned string.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<EventId> for String {
    fn from(value: EventId) -> Self {
        value.0
    }
}

impl AsRef<str> for EventId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for EventId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for EventId {
    type Err = GreenticError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        validate_identifier(value, "EventId")?;
        Ok(Self(value.to_owned()))
    }
}

impl TryFrom<String> for EventId {
    type Error = GreenticError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        EventId::from_str(&value)
    }
}

impl TryFrom<&str> for EventId {
    type Error = GreenticError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        EventId::from_str(value)
    }
}

/// Generic envelope for cross-service events.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct EventEnvelope {
    /// Stable identifier for the event.
    pub id: EventId,
    /// Logical topic for routing (for example `greentic.repo.build.status`).
    pub topic: String,
    /// Fully qualified event type identifier (for example `com.greentic.repo.build.status.v1`).
    pub r#type: String,
    /// Originator of the event (DID, URI, or service identifier).
    pub source: String,
    /// Tenant context propagated with the event.
    pub tenant: TenantCtx,
    /// Optional subject tied to the event (for example `repo:my-service`).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub subject: Option<String>,
    /// Event timestamp in UTC.
    #[cfg_attr(
        feature = "schemars",
        schemars(with = "String", description = "RFC3339 timestamp in UTC")
    )]
    pub time: DateTime<Utc>,
    /// Optional correlation identifier to link related messages.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub correlation_id: Option<String>,
    /// Opaque JSON payload representing the event body.
    pub payload: Value,
    /// Free-form metadata such as idempotency keys.
    #[cfg_attr(feature = "serde", serde(default))]
    pub metadata: EventMetadata,
}
