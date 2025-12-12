//! Session identity and cursor helpers.

use alloc::borrow::ToOwned;
use alloc::string::String;

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{FlowId, TenantCtx};

/// Unique key referencing a persisted session.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct SessionKey(pub String);

impl SessionKey {
    /// Returns the session key as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Creates a new session key from the supplied string.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Generates a random session key using [`uuid`], when enabled.
    #[cfg(feature = "uuid")]
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl From<String> for SessionKey {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for SessionKey {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl core::fmt::Display for SessionKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(feature = "uuid")]
impl From<uuid::Uuid> for SessionKey {
    fn from(value: uuid::Uuid) -> Self {
        Self(value.to_string())
    }
}

const DEFAULT_CANONICAL_ANCHOR: &str = "conversation";
const DEFAULT_CANONICAL_USER: &str = "user";

/// Build the canonical `{tenant}:{provider}:{anchor}:{user}` session key.
///
/// All canonical adapters are expected to follow this format so pause/resume semantics remain
/// deterministic across ingress providers. The anchor defaults to `conversation` and the user
/// defaults to `user` when those fields are not supplied.
pub fn canonical_session_key(
    tenant: impl AsRef<str>,
    provider: impl AsRef<str>,
    anchor: Option<&str>,
    user: Option<&str>,
) -> SessionKey {
    SessionKey::new(format!(
        "{}:{}:{}:{}",
        tenant.as_ref(),
        provider.as_ref(),
        anchor.unwrap_or(DEFAULT_CANONICAL_ANCHOR),
        user.unwrap_or(DEFAULT_CANONICAL_USER)
    ))
}

/// Cursor pointing at a session's position in a flow graph.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct SessionCursor {
    /// Identifier of the node currently owning the session.
    pub node_pointer: String,
    /// Optional wait reason emitted by the node.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub wait_reason: Option<String>,
    /// Optional marker describing pending outbox operations.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub outbox_marker: Option<String>,
}

impl SessionCursor {
    /// Creates a new cursor pointing at the provided node identifier.
    pub fn new(node_pointer: impl Into<String>) -> Self {
        Self {
            node_pointer: node_pointer.into(),
            wait_reason: None,
            outbox_marker: None,
        }
    }

    /// Assigns a wait reason to the cursor.
    pub fn with_wait_reason(mut self, reason: impl Into<String>) -> Self {
        self.wait_reason = Some(reason.into());
        self
    }

    /// Assigns an outbox marker to the cursor.
    pub fn with_outbox_marker(mut self, marker: impl Into<String>) -> Self {
        self.outbox_marker = Some(marker.into());
        self
    }
}

/// Persisted session payload describing how to resume a flow.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct SessionData {
    /// Tenant context associated with the session.
    pub tenant_ctx: TenantCtx,
    /// Flow identifier being executed.
    pub flow_id: FlowId,
    /// Cursor pinpointing where execution paused.
    pub cursor: SessionCursor,
    /// Serialized execution context/state snapshot.
    pub context_json: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_session_key_includes_components() {
        let key = canonical_session_key("tenant", "webhook", Some("room-1"), Some("user-5"));
        assert_eq!(key.as_str(), "tenant:webhook:room-1:user-5");
    }

    #[test]
    fn canonical_session_key_defaults_anchor_and_user() {
        let key = canonical_session_key("tenant", "webhook", None, None);
        assert_eq!(key.as_str(), "tenant:webhook:conversation:user");
    }
}
