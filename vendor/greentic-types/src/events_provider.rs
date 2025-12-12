//! Provider capability descriptors for event fabrics.

use alloc::{string::String, vec::Vec};

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// High-level role of an event provider.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum EventProviderKind {
    /// Message broker capable of publish/subscribe.
    Broker,
    /// Source emitting events into the fabric.
    Source,
    /// Sink consuming events from the fabric.
    Sink,
    /// Bridge translating between fabrics.
    Bridge,
}

/// Transport protocols supported by a provider.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum TransportKind {
    /// NATS transport.
    Nats,
    /// Apache Kafka transport.
    Kafka,
    /// Amazon SQS transport.
    Sqs,
    /// Webhook delivery.
    Webhook,
    /// Email transport.
    Email,
    /// Any other transport not covered above.
    Other(String),
}

/// Delivery guarantees advertised by a provider.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum ReliabilityKind {
    /// At-most-once delivery (lossy).
    AtMostOnce,
    /// At-least-once delivery (duplicates possible).
    AtLeastOnce,
    /// Effectively-once delivery (deduplicated).
    EffectivelyOnce,
}

/// Ordering guarantees for delivered events.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum OrderingKind {
    /// No ordering guarantee.
    None,
    /// Ordering preserved per key/partition.
    PerKey,
    /// Global ordering across all events.
    Global,
}

/// Descriptor capturing the capabilities of an event provider.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct EventProviderDescriptor {
    /// Human-readable name for the provider instance.
    pub name: String,
    /// Provider role (broker, source, sink, bridge).
    pub kind: EventProviderKind,
    /// Transport used by the provider.
    pub transport: TransportKind,
    /// Reliability guarantees.
    pub reliability: ReliabilityKind,
    /// Ordering guarantees.
    pub ordering: OrderingKind,
    /// Optional free-form notes.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub notes: Option<String>,
    /// Tags for discovery and filtering.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub tags: Vec<String>,
}

impl Default for EventProviderDescriptor {
    fn default() -> Self {
        Self {
            name: "provider".into(),
            kind: EventProviderKind::Broker,
            transport: TransportKind::Other("unspecified".into()),
            reliability: ReliabilityKind::AtMostOnce,
            ordering: OrderingKind::None,
            notes: None,
            tags: Vec::new(),
        }
    }
}
