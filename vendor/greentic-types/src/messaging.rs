//! Generic channel messaging envelope shared across providers.

use alloc::{collections::BTreeMap, string::String, vec::Vec};

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::TenantCtx;

/// Collection of metadata entries associated with a channel message.
pub type MessageMetadata = BTreeMap<String, String>;

/// Generic attachment referenced by a channel message.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct Attachment {
    /// MIME type of the attachment (for example `image/png`).
    pub mime_type: String,
    /// URL pointing at the attachment payload.
    pub url: String,
    /// Optional display name for the attachment.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub name: Option<String>,
    /// Optional attachment size in bytes.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub size_bytes: Option<u64>,
}

/// Envelope for channel messages exchanged with adapters.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct ChannelMessageEnvelope {
    /// Stable identifier for the message.
    pub id: String,
    /// Tenant context propagated with the message.
    pub tenant: TenantCtx,
    /// Abstract channel identifier or type.
    pub channel: String,
    /// Conversation or thread identifier.
    pub session_id: String,
    /// Optional user identifier associated with the message.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub user_id: Option<String>,
    /// Optional text content.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub text: Option<String>,
    /// Attachments included with the message.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub attachments: Vec<Attachment>,
    /// Free-form metadata for adapters and flows.
    #[cfg_attr(feature = "serde", serde(default))]
    pub metadata: MessageMetadata,
}
