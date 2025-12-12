//! Storefront, catalog, subscription, and desired state shared models.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    ArtifactRef, BundleId, CollectionId, ComponentRef, DistributorRef, EnvironmentRef,
    MetadataRecordRef, PackId, PackRef, SemverReq, StoreFrontId, StorePlanId, StoreProductId,
    SubscriptionId, TenantCtx,
};

/// Visual theme tokens for a storefront.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct Theme {
    /// Primary color hex code.
    pub primary_color: String,
    /// Secondary color hex code.
    pub secondary_color: String,
    /// Accent color hex code.
    pub accent_color: String,
    /// Background color hex code.
    pub background_color: String,
    /// Text color hex code.
    pub text_color: String,
    /// Primary font family.
    pub font_family: String,
    /// Optional logo URL.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub logo_url: Option<String>,
    /// Optional favicon URL.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub favicon_url: Option<String>,
    /// Optional hero image URL.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub hero_image_url: Option<String>,
    /// Optional hero title.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub hero_title: Option<String>,
    /// Optional hero subtitle.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub hero_subtitle: Option<String>,
    /// Optional card corner radius in pixels.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub card_radius: Option<u8>,
    /// Optional card elevation hint.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub card_elevation: Option<u8>,
    /// Optional button style token.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub button_style: Option<String>,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary_color: "#0f766e".into(),
            secondary_color: "#134e4a".into(),
            accent_color: "#10b981".into(),
            background_color: "#ffffff".into(),
            text_color: "#0f172a".into(),
            font_family: "Inter, sans-serif".into(),
            logo_url: None,
            favicon_url: None,
            hero_image_url: None,
            hero_title: None,
            hero_subtitle: None,
            card_radius: None,
            card_elevation: None,
            button_style: None,
        }
    }
}

/// Layout section kind for storefront composition.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum LayoutSectionKind {
    /// Hero section.
    Hero,
    /// Featured collection of products.
    FeaturedCollection,
    /// Grid of products.
    Grid,
    /// Call-to-action section.
    Cta,
    /// Custom section identified by name.
    Custom(String),
}

/// Layout section configuration.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct LayoutSection {
    /// Stable section identifier.
    pub id: String,
    /// Section kind.
    pub kind: LayoutSectionKind,
    /// Optional collection backing the section.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub collection_id: Option<CollectionId>,
    /// Optional title.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub title: Option<String>,
    /// Optional subtitle.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub subtitle: Option<String>,
    /// Ordering hint for rendering.
    pub sort_order: i32,
    /// Free-form metadata for front-end rendering.
    #[cfg_attr(feature = "serde", serde(default))]
    pub metadata: BTreeMap<String, Value>,
}

/// Collection of products curated for a storefront.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct Collection {
    /// Collection identifier.
    pub id: CollectionId,
    /// Storefront owning the collection.
    pub storefront_id: StoreFrontId,
    /// Display title.
    pub title: String,
    /// Products included in the collection.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub product_ids: Vec<StoreProductId>,
    /// Optional slug.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub slug: Option<String>,
    /// Optional description.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub description: Option<String>,
    /// Sort order hint.
    pub sort_order: i32,
    /// Additional metadata.
    #[cfg_attr(feature = "serde", serde(default))]
    pub metadata: BTreeMap<String, Value>,
}

/// Override applied to a product within a storefront.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct ProductOverride {
    /// Storefront receiving the override.
    pub storefront_id: StoreFrontId,
    /// Product being overridden.
    pub product_id: StoreProductId,
    /// Optional display name override.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub display_name: Option<String>,
    /// Optional short description override.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub short_description: Option<String>,
    /// Badges to render on the product card.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub badges: Vec<String>,
    /// Additional metadata.
    #[cfg_attr(feature = "serde", serde(default))]
    pub metadata: BTreeMap<String, Value>,
}

/// Storefront configuration and content.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct StoreFront {
    /// Storefront identifier.
    pub id: StoreFrontId,
    /// Slug used for routing.
    pub slug: String,
    /// Display name.
    pub name: String,
    /// Visual theme.
    #[cfg_attr(feature = "serde", serde(default))]
    pub theme: Theme,
    /// Layout sections composing the storefront.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub sections: Vec<LayoutSection>,
    /// Curated collections.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub collections: Vec<Collection>,
    /// Product overrides scoped to this storefront.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub overrides: Vec<ProductOverride>,
    /// Optional worker identifier used by messaging.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub worker_id: Option<String>,
    /// Additional metadata.
    #[cfg_attr(feature = "serde", serde(default))]
    pub metadata: BTreeMap<String, Value>,
}

/// Kinds of products exposed by the store catalog.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum StoreProductKind {
    /// Component offering.
    Component,
    /// Flow offering.
    Flow,
    /// Pack offering.
    Pack,
}

/// Strategy used to resolve versions.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum VersionStrategy {
    /// Always track the latest version.
    Latest,
    /// Use a pinned semantic version requirement (legacy shape).
    Pinned {
        /// Version requirement (e.g. ^1.2).
        requirement: SemverReq,
    },
    /// Track a long-term support channel (legacy shape).
    Lts,
    /// Custom strategy identified by name (legacy shape).
    Custom(String),
    /// Always track the latest published version for this component.
    Fixed {
        /// Exact version string (e.g. "1.2.3").
        version: String,
    },
    /// A semver-style range (e.g. ">=1.2,<2.0").
    Range {
        /// Version range expression.
        range: String,
    },
    /// A named channel (e.g. "stable", "beta", "canary").
    Channel {
        /// Channel name.
        channel: String,
    },
    /// Forward-compatible escape hatch for unknown strategies.
    CustomTagged {
        /// Free-form value for the strategy.
        value: String,
    },
}

#[cfg(feature = "serde")]
impl Serialize for VersionStrategy {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        #[serde(rename_all = "snake_case")]
        enum Legacy<'a> {
            Latest,
            Pinned { requirement: &'a SemverReq },
            Lts,
            Custom(&'a str),
        }

        #[derive(Serialize)]
        struct Tagged<'a> {
            #[serde(rename = "kind")]
            kind: &'static str,
            #[serde(skip_serializing_if = "Option::is_none")]
            version: Option<&'a String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            range: Option<&'a String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            channel: Option<&'a String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            value: Option<&'a String>,
        }

        match self {
            VersionStrategy::Latest => Legacy::Latest.serialize(serializer),
            VersionStrategy::Pinned { requirement } => {
                Legacy::Pinned { requirement }.serialize(serializer)
            }
            VersionStrategy::Lts => Legacy::Lts.serialize(serializer),
            VersionStrategy::Custom(value) => Legacy::Custom(value).serialize(serializer),
            VersionStrategy::Fixed { version } => Tagged {
                kind: "fixed",
                version: Some(version),
                range: None,
                channel: None,
                value: None,
            }
            .serialize(serializer),
            VersionStrategy::Range { range } => Tagged {
                kind: "range",
                version: None,
                range: Some(range),
                channel: None,
                value: None,
            }
            .serialize(serializer),
            VersionStrategy::Channel { channel } => Tagged {
                kind: "channel",
                version: None,
                range: None,
                channel: Some(channel),
                value: None,
            }
            .serialize(serializer),
            VersionStrategy::CustomTagged { value } => Tagged {
                kind: "custom",
                version: None,
                range: None,
                channel: None,
                value: Some(value),
            }
            .serialize(serializer),
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for VersionStrategy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "snake_case")]
        enum Legacy {
            Latest,
            Pinned { requirement: SemverReq },
            Lts,
            Custom(String),
        }

        #[derive(Deserialize)]
        #[serde(tag = "kind", rename_all = "snake_case")]
        enum Tagged {
            Latest,
            Fixed { version: String },
            Range { range: String },
            Channel { channel: String },
            Custom { value: String },
        }

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Wrapper {
            Tagged(Tagged),
            Legacy(Legacy),
        }

        match Wrapper::deserialize(deserializer)? {
            Wrapper::Tagged(tagged) => match tagged {
                Tagged::Latest => Ok(VersionStrategy::Latest),
                Tagged::Fixed { version } => Ok(VersionStrategy::Fixed { version }),
                Tagged::Range { range } => Ok(VersionStrategy::Range { range }),
                Tagged::Channel { channel } => Ok(VersionStrategy::Channel { channel }),
                Tagged::Custom { value } => Ok(VersionStrategy::CustomTagged { value }),
            },
            Wrapper::Legacy(legacy) => match legacy {
                Legacy::Latest => Ok(VersionStrategy::Latest),
                Legacy::Pinned { requirement } => Ok(VersionStrategy::Pinned { requirement }),
                Legacy::Lts => Ok(VersionStrategy::Lts),
                Legacy::Custom(value) => Ok(VersionStrategy::Custom(value)),
            },
        }
    }
}

/// Map of capability group -> list of capability values.
pub type CapabilityMap = BTreeMap<String, Vec<String>>;

/// Catalog product describing a component, flow, or pack.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct StoreProduct {
    /// Product identifier.
    pub id: StoreProductId,
    /// Product kind.
    pub kind: StoreProductKind,
    /// Display name.
    pub name: String,
    /// Slug for routing.
    pub slug: String,
    /// Description.
    pub description: String,
    /// Source repository reference.
    pub source_repo: crate::RepoRef,
    /// Optional component reference.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub component_ref: Option<ComponentRef>,
    /// Optional pack reference.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub pack_ref: Option<PackId>,
    /// Optional category label.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub category: Option<String>,
    /// Tags for filtering.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub tags: Vec<String>,
    /// Capabilities exposed by the product.
    #[cfg_attr(feature = "serde", serde(default))]
    pub capabilities: CapabilityMap,
    /// Version resolution strategy.
    pub version_strategy: VersionStrategy,
    /// Default plan identifier, if any.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub default_plan_id: Option<StorePlanId>,
    /// Convenience flag indicating the default plan is free.
    pub is_free: bool,
    /// Additional metadata.
    #[cfg_attr(feature = "serde", serde(default))]
    pub metadata: BTreeMap<String, Value>,
}

/// Pricing model for a plan.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum PriceModel {
    /// Free plan.
    Free,
    /// Flat recurring price.
    Flat {
        /// Amount in micro-units per period.
        amount_micro: u64,
        /// Billing period length in days.
        period_days: u16,
    },
    /// Metered pricing with included units.
    Metered {
        /// Included units per period.
        included_units: u64,
        /// Overage rate per additional unit (micro-units).
        overage_rate_micro: u64,
        /// Unit label (for example `build-minute`).
        unit_label: String,
    },
    /// Enterprise/custom pricing.
    Enterprise {
        /// Human-readable description.
        description: String,
    },
}

/// Plan limits used for entitlements.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct PlanLimits {
    /// Maximum environments allowed.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub max_environments: Option<u32>,
    /// Maximum subscriptions allowed.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub max_subscriptions: Option<u32>,
    /// Included units per period (semantic depends on product).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub monthly_units_included: Option<u64>,
    /// Additional metadata.
    #[cfg_attr(feature = "serde", serde(default))]
    pub metadata: BTreeMap<String, Value>,
}

/// Plan associated with a store product.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct StorePlan {
    /// Plan identifier.
    pub id: StorePlanId,
    /// Plan name.
    pub name: String,
    /// Plan description.
    pub description: String,
    /// Pricing model.
    pub price_model: PriceModel,
    /// Plan limits.
    #[cfg_attr(feature = "serde", serde(default))]
    pub limits: PlanLimits,
    /// Tags for classification.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub tags: Vec<String>,
    /// Additional metadata.
    #[cfg_attr(feature = "serde", serde(default))]
    pub metadata: BTreeMap<String, Value>,
}

/// Subscription lifecycle status.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum SubscriptionStatus {
    /// Draft subscription (pending approval).
    Draft,
    /// Active subscription.
    Active,
    /// Paused subscription.
    Paused,
    /// Cancelled subscription.
    Cancelled,
    /// Subscription encountered an error.
    Error,
}

/// Subscription entry linking a tenant to a product and plan.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct Subscription {
    /// Subscription identifier.
    pub id: SubscriptionId,
    /// Tenant context owning the subscription.
    pub tenant_ctx: TenantCtx,
    /// Product identifier.
    pub product_id: StoreProductId,
    /// Plan identifier.
    pub plan_id: StorePlanId,
    /// Optional target environment.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub environment_ref: Option<EnvironmentRef>,
    /// Optional distributor responsible for the environment.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub distributor_ref: Option<DistributorRef>,
    /// Current status.
    pub status: SubscriptionStatus,
    /// Additional metadata.
    #[cfg_attr(feature = "serde", serde(default))]
    pub metadata: BTreeMap<String, Value>,
}

/// Choice between component or pack reference.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum PackOrComponentRef {
    /// Component reference.
    Component(ComponentRef),
    /// Pack reference.
    Pack(PackId),
}

/// Selector describing whether a component or pack should be deployed.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum ArtifactSelector {
    /// Component reference.
    Component(ComponentRef),
    /// Pack reference.
    Pack(PackRef),
}

/// Desired subscription entry supplied to the distributor.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct DesiredSubscriptionEntry {
    /// Target artifact selection.
    pub selector: ArtifactSelector,
    /// Version strategy to apply.
    pub version_strategy: VersionStrategy,
    /// Configuration overrides.
    #[cfg_attr(feature = "serde", serde(default))]
    pub config_overrides: BTreeMap<String, Value>,
    /// Policy tags for downstream enforcement.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub policy_tags: Vec<String>,
    /// Additional metadata.
    #[cfg_attr(feature = "serde", serde(default))]
    pub metadata: BTreeMap<String, Value>,
}

/// Desired state for an environment.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct DesiredState {
    /// Tenant context owning the desired state.
    pub tenant: TenantCtx,
    /// Target environment reference.
    pub environment_ref: EnvironmentRef,
    /// Desired subscriptions.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub entries: Vec<DesiredSubscriptionEntry>,
    /// Desired state version.
    pub version: u64,
    /// Additional metadata.
    #[cfg_attr(feature = "serde", serde(default))]
    pub metadata: BTreeMap<String, Value>,
}

/// Connection kind for an environment.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum ConnectionKind {
    /// Online environment with direct connectivity.
    Online,
    /// Offline or air-gapped environment.
    Offline,
}

/// Environment registry entry.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct Environment {
    /// Environment identifier.
    pub id: EnvironmentRef,
    /// Tenant context owning the environment.
    pub tenant: TenantCtx,
    /// Human-readable name.
    pub name: String,
    /// Labels for selection and grouping.
    #[cfg_attr(feature = "serde", serde(default))]
    pub labels: BTreeMap<String, String>,
    /// Distributor responsible for this environment.
    pub distributor_ref: DistributorRef,
    /// Connection kind.
    pub connection_kind: ConnectionKind,
    /// Additional metadata.
    #[cfg_attr(feature = "serde", serde(default))]
    pub metadata: BTreeMap<String, Value>,
}

impl Environment {
    /// Constructs a new environment with the required identifiers.
    pub fn new(
        id: EnvironmentRef,
        tenant: TenantCtx,
        distributor_ref: DistributorRef,
        connection_kind: ConnectionKind,
        name: impl Into<String>,
    ) -> Self {
        Self {
            id,
            tenant,
            name: name.into(),
            distributor_ref,
            connection_kind,
            labels: BTreeMap::new(),
            metadata: BTreeMap::new(),
        }
    }
}

/// Rollout lifecycle state for an environment.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum RolloutState {
    /// Rollout is pending scheduling or validation.
    Pending,
    /// Rollout plan generation is in progress.
    Planning,
    /// Rollout is actively executing.
    InProgress,
    /// Rollout completed successfully.
    Succeeded,
    /// Rollout failed.
    Failed,
    /// Rollout is blocked (for example policy or compliance).
    Blocked,
}

/// Status record for an environment rollout.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct RolloutStatus {
    /// Target environment.
    pub environment_ref: EnvironmentRef,
    /// Desired state version associated with this rollout.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub desired_state_version: Option<u64>,
    /// Current rollout state.
    pub state: RolloutState,
    /// Optional bundle used for offline rollouts.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub bundle_id: Option<BundleId>,
    /// Optional human-readable message.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub message: Option<String>,
    /// Additional metadata.
    #[cfg_attr(feature = "serde", serde(default))]
    pub metadata: BTreeMap<String, Value>,
}

/// Bundle specification for offline or air-gapped deployments.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct BundleSpec {
    /// Identifier of the distribution-bundle `.gtpack` (pack id).
    pub bundle_id: BundleId,
    /// Tenant context for the bundle.
    pub tenant: TenantCtx,
    /// Target environment.
    pub environment_ref: EnvironmentRef,
    /// Version of the desired state used to construct the bundle.
    pub desired_state_version: u64,
    /// Artifact references included in the bundle.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub artifact_refs: Vec<ArtifactRef>,
    /// Metadata record references (SBOMs, attestations, signatures).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub metadata_refs: Vec<MetadataRecordRef>,
    /// Additional metadata.
    #[cfg_attr(feature = "serde", serde(default))]
    pub additional_metadata: BTreeMap<String, Value>,
}

/// Export specification used to request a bundle from a desired state.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct DesiredStateExportSpec {
    /// Tenant context owning the desired state.
    pub tenant: TenantCtx,
    /// Target environment.
    pub environment_ref: EnvironmentRef,
    /// Desired state version to export.
    pub desired_state_version: u64,
    /// Whether to include artifacts in the bundle.
    #[cfg_attr(feature = "serde", serde(default))]
    pub include_artifacts: bool,
    /// Whether to include metadata (SBOMs, attestations).
    #[cfg_attr(feature = "serde", serde(default))]
    pub include_metadata: bool,
    /// Additional metadata.
    #[cfg_attr(feature = "serde", serde(default))]
    pub metadata: BTreeMap<String, Value>,
}
