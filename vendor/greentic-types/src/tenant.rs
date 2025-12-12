//! Tenant-centric identity helpers.

use alloc::collections::BTreeMap;
use alloc::string::String;

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{TeamId, TenantContext, TenantCtx, TenantId, UserId};

/// Metadata describing an impersonated user acting on behalf of the main identity.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct Impersonation {
    /// Identifier of the user performing the impersonation.
    pub actor_id: UserId,
    /// Optional justification recorded for auditing.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub reason: Option<String>,
}

/// Stable multi-tenant identity extracted from [`TenantCtx`].
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct TenantIdentity {
    /// Tenant identifier.
    pub tenant_id: TenantId,
    /// Optional team identifier.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub team_id: Option<TeamId>,
    /// Optional user identifier.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub user_id: Option<UserId>,
    /// Optional impersonation information.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub impersonation: Option<Impersonation>,
    /// Free-form attributes propagated for routing and tracing.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "BTreeMap::is_empty")
    )]
    pub attributes: BTreeMap<String, String>,
}

impl TenantIdentity {
    /// Creates a new tenant identity scoped to a tenant id.
    pub fn new(tenant_id: TenantId) -> Self {
        Self {
            tenant_id,
            team_id: None,
            user_id: None,
            impersonation: None,
            attributes: BTreeMap::new(),
        }
    }
}

impl From<&TenantCtx> for TenantIdentity {
    fn from(ctx: &TenantCtx) -> Self {
        Self {
            tenant_id: ctx.tenant_id.clone(),
            team_id: ctx.team_id.clone().or_else(|| ctx.team.clone()),
            user_id: ctx.user_id.clone().or_else(|| ctx.user.clone()),
            impersonation: ctx.impersonation.clone(),
            attributes: ctx.attributes.clone(),
        }
    }
}

impl TenantCtx {
    /// Returns the tenant identity derived from this context.
    pub fn identity(&self) -> TenantIdentity {
        TenantIdentity::from(self)
    }

    /// Returns the lightweight tenant context shared with tooling.
    pub fn tenant_context(&self) -> TenantContext {
        TenantContext::from(self)
    }

    /// Returns the impersonation context, when present.
    pub fn impersonated_by(&self) -> Option<&Impersonation> {
        self.impersonation.as_ref()
    }

    /// Updates the identity fields to match the provided value.
    pub fn with_identity(mut self, identity: TenantIdentity) -> Self {
        self.tenant = identity.tenant_id.clone();
        self.tenant_id = identity.tenant_id;
        self.team = identity.team_id.clone();
        self.team_id = identity.team_id;
        self.user = identity.user_id.clone();
        self.user_id = identity.user_id;
        self.impersonation = identity.impersonation;
        self.attributes = identity.attributes;
        self
    }
}
