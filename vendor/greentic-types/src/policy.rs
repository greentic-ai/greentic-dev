//! Network policy primitives.

use alloc::string::String;
use alloc::vec::Vec;

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Network protocols supported by allow lists.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum Protocol {
    /// Hypertext Transfer Protocol.
    Http,
    /// Hypertext Transfer Protocol Secure.
    Https,
    /// Generic TCP connectivity.
    Tcp,
    /// Generic UDP connectivity.
    Udp,
    /// gRPC.
    Grpc,
    /// Any protocol not covered above.
    Custom(String),
}

/// Allow list describing permitted domains, ports, and protocols.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct AllowList {
    /// Allowed domain suffixes or exact hosts.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub domains: Vec<String>,
    /// Allowed port numbers.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub ports: Vec<u16>,
    /// Allowed network protocols.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub protocols: Vec<Protocol>,
}

impl AllowList {
    /// Creates an empty allow list.
    pub fn empty() -> Self {
        Self {
            domains: Vec::new(),
            ports: Vec::new(),
            protocols: Vec::new(),
        }
    }

    /// Returns `true` when the allow list contains no rules.
    pub fn is_empty(&self) -> bool {
        self.domains.is_empty() && self.ports.is_empty() && self.protocols.is_empty()
    }
}

impl Default for AllowList {
    fn default() -> Self {
        Self::empty()
    }
}

/// High-level network policy composed of allow lists.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct NetworkPolicy {
    /// Allow list enforced for egress connections.
    pub egress: AllowList,
    /// Whether destinations not present in the allow list should be denied.
    pub deny_on_miss: bool,
}

impl NetworkPolicy {
    /// Creates a policy denying unknown destinations by default.
    pub fn strict(egress: AllowList) -> Self {
        Self {
            egress,
            deny_on_miss: true,
        }
    }
}

/// Result of evaluating a network policy.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct PolicyDecision {
    /// Canonical status for the decision.
    pub status: PolicyDecisionStatus,
    /// Optional list of reasons.
    pub reasons: Vec<String>,
    /// Legacy allow flag (retained for backward compatibility).
    pub allow: Option<bool>,
    /// Legacy single reason (retained for backward compatibility).
    pub reason: Option<String>,
}

/// Status for a policy decision.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum PolicyDecisionStatus {
    /// Request is allowed.
    Allow,
    /// Request is denied.
    Deny,
}

#[cfg(feature = "serde")]
mod serde_impls {
    use super::{PolicyDecision, PolicyDecisionStatus};
    use alloc::vec::Vec;
    use serde::de::{self, MapAccess, Visitor};
    use serde::ser::SerializeStruct;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    impl Serialize for PolicyDecision {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            // status + reasons always emitted; legacy fields only when present.
            let mut len = 2;
            if self.allow.is_some() {
                len += 1;
            }
            if self.reason.is_some() {
                len += 1;
            }
            let mut state = serializer.serialize_struct("PolicyDecision", len)?;
            state.serialize_field("status", &self.status)?;
            state.serialize_field("reasons", &self.reasons)?;
            if let Some(allow) = &self.allow {
                state.serialize_field("allow", allow)?;
            }
            if let Some(reason) = &self.reason {
                state.serialize_field("reason", reason)?;
            }
            state.end()
        }
    }

    impl<'de> Deserialize<'de> for PolicyDecision {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            enum Field {
                Allow,
                Reason,
                Status,
                Reasons,
                Unknown,
            }

            impl<'de> Deserialize<'de> for Field {
                fn deserialize<D2>(deserializer: D2) -> Result<Self, D2::Error>
                where
                    D2: Deserializer<'de>,
                {
                    struct FieldVisitor;

                    impl<'de> Visitor<'de> for FieldVisitor {
                        type Value = Field;

                        fn expecting(
                            &self,
                            formatter: &mut core::fmt::Formatter,
                        ) -> core::fmt::Result {
                            formatter.write_str("`allow`, `reason`, `status`, or `reasons`")
                        }

                        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                        where
                            E: de::Error,
                        {
                            Ok(match value {
                                "allow" => Field::Allow,
                                "reason" => Field::Reason,
                                "status" => Field::Status,
                                "reasons" => Field::Reasons,
                                _ => Field::Unknown,
                            })
                        }
                    }

                    deserializer.deserialize_identifier(FieldVisitor)
                }
            }

            struct PolicyDecisionVisitor;

            impl<'de> Visitor<'de> for PolicyDecisionVisitor {
                type Value = PolicyDecision;

                fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                    formatter.write_str("policy decision")
                }

                fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
                where
                    M: MapAccess<'de>,
                {
                    let mut allow: Option<Option<bool>> = None;
                    let mut reason: Option<Option<String>> = None;
                    let mut status: Option<PolicyDecisionStatus> = None;
                    let mut reasons: Option<Vec<String>> = None;

                    while let Some(key) = map.next_key()? {
                        match key {
                            Field::Allow => {
                                if allow.is_some() {
                                    return Err(de::Error::duplicate_field("allow"));
                                }
                                allow = Some(map.next_value()?);
                            }
                            Field::Reason => {
                                if reason.is_some() {
                                    return Err(de::Error::duplicate_field("reason"));
                                }
                                reason = Some(map.next_value()?);
                            }
                            Field::Status => {
                                if status.is_some() {
                                    return Err(de::Error::duplicate_field("status"));
                                }
                                status = Some(map.next_value()?);
                            }
                            Field::Reasons => {
                                if reasons.is_some() {
                                    return Err(de::Error::duplicate_field("reasons"));
                                }
                                reasons = Some(map.next_value()?);
                            }
                            Field::Unknown => {
                                // Ignore unknown fields for forward compatibility.
                                let _ = map.next_value::<de::IgnoredAny>()?;
                            }
                        }
                    }

                    let status = status
                        .or_else(|| {
                            allow.flatten().map(|flag| {
                                if flag {
                                    PolicyDecisionStatus::Allow
                                } else {
                                    PolicyDecisionStatus::Deny
                                }
                            })
                        })
                        .unwrap_or(PolicyDecisionStatus::Allow);

                    let reasons_vec = match (reasons, reason.clone()) {
                        (Some(list), _) => list,
                        (None, Some(Some(msg))) => alloc::vec![msg],
                        (None, _) => Vec::new(),
                    };

                    Ok(PolicyDecision {
                        status,
                        reasons: reasons_vec,
                        allow: allow.flatten(),
                        reason: reason.flatten(),
                    })
                }
            }

            deserializer.deserialize_struct(
                "PolicyDecision",
                &["status", "reasons", "allow", "reason"],
                PolicyDecisionVisitor,
            )
        }
    }
}
