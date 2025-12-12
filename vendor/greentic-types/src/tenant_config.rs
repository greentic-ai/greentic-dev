//! Tenant-facing configuration document shapes (skin/auth/config/did).
//!
//! These structs mirror the JSON documents served to the Loveable UI. They intentionally avoid
//! hard-coding UI navigation semantics (tabs, slots, etc.) to keep the types crate forward
//! compatible.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Branding and layout configuration for a tenant (`skin.json`).
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct RepoSkin {
    /// Tenant identifier the skin belongs to.
    pub tenant_id: String,
    /// Optional human-readable tenant name.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub tenant_name: Option<String>,
    /// Optional product name to display.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub product_name: Option<String>,
    /// Theme configuration (colors, fonts, imagery).
    pub theme: RepoSkinTheme,
    /// Optional layout flags controlling navigation visibility and hero band.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub layout: Option<RepoSkinLayout>,
    /// Optional worker panel configuration.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub worker_panel: Option<RepoWorkerPanel>,
    /// Optional tenant links for docs/support/status.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub links: Option<RepoSkinLinks>,
}

/// Theme settings for login and console surfaces.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct RepoSkinTheme {
    /// Primary logo URL.
    pub logo_url: String,
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
    /// Primary brand color.
    pub primary_color: String,
    /// Accent brand color.
    pub accent_color: String,
    /// Optional background color.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub background_color: Option<String>,
    /// Optional background gradient.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub background_gradient: Option<String>,
    /// Optional font family override.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub font_family: Option<String>,
    /// Optional success color override.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub success_color: Option<String>,
    /// Optional warning color override.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub warning_color: Option<String>,
    /// Optional danger color override.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub danger_color: Option<String>,
}

/// Layout toggles describing visible console sections.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct RepoSkinLayout {
    /// Whether to show the dashboard tab.
    pub show_dashboard: bool,
    /// Whether to show the repositories tab.
    pub show_repositories: bool,
    /// Whether to show the pipeline tab.
    pub show_pipeline: bool,
    /// Whether to show the packs tab.
    pub show_packs: bool,
    /// Whether to show the trust & access tab.
    pub show_trust_access: bool,
    /// Whether to show the audit & compliance tab.
    pub show_audit_compliance: bool,
    /// Whether to show the admin/config tab.
    pub show_admin_config: Option<bool>,
    /// Whether to render the hero band.
    pub show_hero_band: Option<bool>,
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
}

/// Worker panel placement and defaults.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct RepoWorkerPanel {
    /// Whether the worker panel is enabled.
    pub enabled: bool,
    /// Optional display title.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub title: Option<String>,
    /// Whether the panel should open by default.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub default_open: Option<bool>,
    /// Preferred position (for example `left` or `right`).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub position: Option<String>,
}

/// Optional tenant links for navigation.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct RepoSkinLinks {
    /// Optional documentation URL.
    pub docs_url: Option<String>,
    /// Optional support URL.
    pub support_url: Option<String>,
    /// Optional status page URL.
    pub status_url: Option<String>,
}

/// Login options for a tenant (`auth.json`).
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct RepoAuth {
    /// Tenant identifier the auth config belongs to.
    pub tenant_id: String,
    /// Available identity providers.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub identity_providers: Vec<IdentityProviderOption>,
}

/// Identity provider option shown on the login page.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct IdentityProviderOption {
    /// Provider pack identifier.
    pub id: String,
    /// Optional kind hint (for example `identity-provider`).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub kind: Option<String>,
    /// Button label.
    pub label: String,
    /// Optional icon identifier.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub icon: Option<String>,
    /// Optional button style.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub button_style: Option<String>,
    /// Optional ordering hint.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub order: Option<i32>,
    /// Login URL for the provider.
    pub login_url: String,
    /// Optional description text.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub description: Option<String>,
    /// Whether the provider is recommended.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub recommended: Option<bool>,
}

/// Tenant UI configuration exposed to the console (`config.json`).
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct RepoTenantConfig {
    /// Tenant identifier the config belongs to.
    pub tenant_id: String,
    /// Structural flags: active sections for the UI. Values are conventions, not enforced here.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub enabled_tabs: Vec<String>,
    /// Enabled packs grouped by kind.
    pub enabled_packs: EnabledPacks,
    /// Default pipeline choices.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub default_pipeline: Option<DefaultPipeline>,
    /// Optional configured stores.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub stores: Option<Vec<StoreTarget>>,
    /// Optional configured distributors.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub distributors: Option<Vec<DistributorTarget>>,
    /// Feature flags for the UI.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub features: Option<RepoConfigFeatures>,
    /// Maps page slots to UI action handler pack identifiers.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub page_handlers: Option<BTreeMap<String, String>>,
}

/// Enabled packs grouped by capability.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct EnabledPacks {
    /// Identity provider pack identifiers.
    pub identity_providers: Option<Vec<String>>,
    /// Source provider pack identifiers.
    pub source_providers: Option<Vec<String>>,
    /// Scanner pack identifiers.
    pub scanners: Option<Vec<String>>,
    /// Signing pack identifiers.
    pub signing: Option<Vec<String>>,
    /// Attestation pack identifiers.
    pub attestation: Option<Vec<String>>,
    /// Policy engine pack identifiers.
    pub policy_engines: Option<Vec<String>>,
    /// OCI provider pack identifiers.
    pub oci_providers: Option<Vec<String>>,
}

/// Default pipeline selections per capability.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct DefaultPipeline {
    /// Default scanners to run.
    pub scanners: Option<Vec<String>>,
    /// Default signing provider identifier.
    pub signing: Option<String>,
    /// Default attestation provider identifier.
    pub attestation: Option<String>,
    /// Default policy engine identifier.
    pub policy_engine: Option<String>,
    /// Default OCI provider identifier.
    pub oci_provider: Option<String>,
}

/// Public store target descriptor.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct StoreTarget {
    /// Store identifier.
    pub id: String,
    /// Human-readable label.
    pub label: String,
    /// Store URL.
    pub url: String,
    /// Optional description.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub description: Option<String>,
}

/// Public distributor target descriptor.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct DistributorTarget {
    /// Distributor identifier.
    pub id: String,
    /// Human-readable label.
    pub label: String,
    /// Distributor URL.
    pub url: String,
    /// Optional description.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub description: Option<String>,
}

/// Feature flags for the UI.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct RepoConfigFeatures {
    /// Whether manual approvals are allowed.
    pub allow_manual_approve: Option<bool>,
    /// Whether to show advanced scan views.
    pub show_advanced_scan_views: Option<bool>,
    /// Whether to surface experimental modules.
    pub show_experimental_modules: Option<bool>,
}

/// DID document used for tenant discovery (`did.json`).
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct TenantDidDocument {
    /// Raw @context, preserved as provided.
    #[cfg_attr(feature = "serde", serde(rename = "@context", default))]
    pub raw_context: Option<DidContext>,
    /// Document identifier (did:web).
    pub id: String,
    /// Verification methods (optional).
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "verificationMethod",
            default,
            skip_serializing_if = "Option::is_none"
        )
    )]
    pub verification_method: Option<Vec<VerificationMethod>>,
    /// Authentication references (optional).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub authentication: Option<Vec<String>>,
    /// Service endpoints.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub service: Vec<DidService>,
}

impl TenantDidDocument {
    /// Returns the normalized `@context` value as a vector of strings.
    pub fn context(&self) -> Vec<String> {
        match &self.raw_context {
            Some(DidContext::Single(value)) => alloc::vec![value.clone()],
            Some(DidContext::Multiple(values)) => values.clone(),
            None => Vec::new(),
        }
    }
}

/// @context representation supporting single string or array.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum DidContext {
    /// Single string context.
    Single(String),
    /// Array of contexts.
    Multiple(Vec<String>),
}

/// Verification method descriptor within a DID document.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct VerificationMethod {
    /// Identifier for the verification method.
    pub id: String,
    /// Type of verification method.
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub r#type: String,
    /// Controller of the verification method.
    pub controller: String,
    /// Optional JWK.
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "publicKeyJwk",
            default,
            skip_serializing_if = "Option::is_none"
        )
    )]
    pub public_key_jwk: Option<Value>,
    /// Optional multibase key.
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "publicKeyMultibase",
            default,
            skip_serializing_if = "Option::is_none"
        )
    )]
    pub public_key_multibase: Option<String>,
}

/// Service endpoint descriptor within a DID document.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct DidService {
    /// Service identifier.
    pub id: String,
    /// Service type.
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub r#type: String,
    /// Service endpoint URL.
    #[cfg_attr(feature = "serde", serde(rename = "serviceEndpoint"))]
    pub service_endpoint: String,
}
