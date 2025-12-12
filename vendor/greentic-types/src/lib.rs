#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![warn(clippy::unwrap_used, clippy::expect_used)]

//! Shared types and helpers for Greentic multi-tenant flows.
//!
//! # Overview
//! Greentic components share a single crate for tenancy, execution outcomes, network limits, and
//! schema metadata. Use the strongly-typed identifiers to keep flows, packs, and components
//! consistent across repositories and to benefit from serde + schema validation automatically.
//!
//! ## Tenant contexts
//! ```
//! use greentic_types::{EnvId, TenantCtx, TenantId};
//!
//! let ctx = TenantCtx::new("prod".parse().unwrap(), "tenant-42".parse().unwrap())
//!     .with_team(Some("team-ops".parse().unwrap()))
//!     .with_user(Some("agent-007".parse().unwrap()));
//! assert_eq!(ctx.tenant_id.as_str(), "tenant-42");
//! ```
//!
//! ## Run results & serialization
//! ```
//! # #[cfg(feature = "time")] {
//! use greentic_types::{FlowId, PackId, RunResult, RunStatus, SessionKey};
//! use semver::Version;
//! use time::OffsetDateTime;
//!
//! let now = OffsetDateTime::UNIX_EPOCH;
//! let result = RunResult {
//!     session_id: SessionKey::from("sess-1"),
//!     pack_id: "greentic.demo.pack".parse().unwrap(),
//!     pack_version: Version::parse("1.0.0").unwrap(),
//!     flow_id: "demo-flow".parse().unwrap(),
//!     started_at_utc: now,
//!     finished_at_utc: now,
//!     status: RunStatus::Success,
//!     node_summaries: Vec::new(),
//!     failures: Vec::new(),
//!     artifacts_dir: None,
//! };
//! println!("{}", serde_json::to_string_pretty(&result).unwrap());
//! # }
//! ```
//!
//! Published JSON Schemas are listed in [`SCHEMAS.md`](SCHEMAS.md) and hosted under
//! <https://greentic-ai.github.io/greentic-types/schemas/v1/>.

extern crate alloc;

/// Crate version string exposed for telemetry and capability negotiation.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// Base URL for all published JSON Schemas.
pub const SCHEMA_BASE_URL: &str = "https://greentic-ai.github.io/greentic-types/schemas/v1";

pub mod bindings;
pub mod capabilities;
#[cfg(feature = "std")]
pub mod cbor;
pub mod component;
pub mod deployment;
pub mod distributor;
pub mod events;
pub mod events_provider;
pub mod flow;
pub mod messaging;
pub mod pack_manifest;
pub mod store;
pub mod supply_chain;
pub mod worker;

pub mod context;
pub mod error;
pub mod outcome;
pub mod pack;
pub mod policy;
pub mod run;
#[cfg(all(feature = "schemars", feature = "std"))]
pub mod schema;
pub mod session;
pub mod state;
pub mod telemetry;
pub mod tenant;
pub mod tenant_config;

pub use bindings::hints::{
    BindingsHints, EnvHints, McpHints, McpServer, NetworkHints, SecretsHints,
};
pub use capabilities::{
    Capabilities, FsCaps, HttpCaps, KvCaps, Limits, NetCaps, SecretsCaps, TelemetrySpec, ToolsCaps,
};
#[cfg(feature = "std")]
pub use cbor::{CborError, decode_pack_manifest, encode_pack_manifest};
pub use component::{
    ComponentCapabilities, ComponentConfigurators, ComponentManifest, ComponentOperation,
    ComponentProfileError, ComponentProfiles, EnvCapabilities, EventsCapabilities,
    FilesystemCapabilities, FilesystemMode, FilesystemMount, HostCapabilities, HttpCapabilities,
    IaCCapabilities, MessagingCapabilities, ResourceHints, SecretsCapabilities, StateCapabilities,
    TelemetryCapabilities, TelemetryScope, WasiCapabilities,
};
pub use context::{Cloud, DeploymentCtx, Platform};
pub use deployment::{
    ChannelPlan, DeploymentPlan, MessagingPlan, MessagingSubjectPlan, OAuthPlan, RunnerPlan,
    SecretPlan, TelemetryPlan,
};
pub use distributor::{
    ArtifactLocation, CacheInfo, ComponentDigest, ComponentStatus, DistributorEnvironmentId,
    ResolveComponentRequest, ResolveComponentResponse, SignatureSummary,
};
pub use error::{ErrorCode, GResult, GreenticError};
pub use events::{EventEnvelope, EventId, EventMetadata};
pub use events_provider::{
    EventProviderDescriptor, EventProviderKind, OrderingKind, ReliabilityKind, TransportKind,
};
pub use flow::{
    ComponentRef as FlowComponentRef, Flow, FlowKind, FlowMetadata, InputMapping, Node,
    OutputMapping, Routing, TelemetryHints,
};
pub use messaging::{Attachment, ChannelMessageEnvelope, MessageMetadata};
pub use outcome::Outcome;
pub use pack::{PackRef, Signature, SignatureAlgorithm};
pub use pack_manifest::{
    ComponentCapability, PackDependency, PackFlowEntry, PackKind, PackManifest, PackSignatures,
};
pub use policy::{AllowList, NetworkPolicy, PolicyDecision, PolicyDecisionStatus, Protocol};
#[cfg(feature = "time")]
pub use run::RunResult;
pub use run::{NodeFailure, NodeStatus, NodeSummary, RunStatus, TranscriptOffset};
pub use session::canonical_session_key;
pub use session::{SessionCursor, SessionData, SessionKey};
pub use state::{StateKey, StatePath};
pub use store::{
    ArtifactSelector, BundleSpec, CapabilityMap, Collection, ConnectionKind, DesiredState,
    DesiredStateExportSpec, DesiredSubscriptionEntry, Environment, LayoutSection,
    LayoutSectionKind, PackOrComponentRef, PlanLimits, PriceModel, ProductOverride, RolloutState,
    RolloutStatus, StoreFront, StorePlan, StoreProduct, StoreProductKind, Subscription,
    SubscriptionStatus, Theme, VersionStrategy,
};
pub use supply_chain::{
    AttestationStatement, BuildPlan, BuildStatus, BuildStatusKind, MetadataRecord, PredicateType,
    RepoContext, ScanKind, ScanRequest, ScanResult, ScanStatusKind, SignRequest, StoreContext,
    VerifyRequest, VerifyResult,
};
#[cfg(feature = "otel-keys")]
pub use telemetry::OtlpKeys;
pub use telemetry::SpanContext;
#[cfg(feature = "telemetry-autoinit")]
pub use telemetry::TelemetryCtx;
pub use tenant::{Impersonation, TenantIdentity};
pub use tenant_config::{
    DefaultPipeline, DidContext, DidService, DistributorTarget, EnabledPacks,
    IdentityProviderOption, RepoAuth, RepoConfigFeatures, RepoSkin, RepoSkinLayout, RepoSkinLinks,
    RepoSkinTheme, RepoTenantConfig, RepoWorkerPanel, StoreTarget, TenantDidDocument,
    VerificationMethod,
};
pub use worker::{WorkerMessage, WorkerRequest, WorkerResponse};

#[cfg(feature = "schemars")]
use alloc::borrow::Cow;
use alloc::{borrow::ToOwned, collections::BTreeMap, format, string::String, vec::Vec};
use core::fmt;
use core::str::FromStr;
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use semver::VersionReq;
#[cfg(feature = "time")]
use time::OffsetDateTime;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "std")]
use alloc::boxed::Box;

#[cfg(feature = "std")]
use std::error::Error as StdError;

/// Validates identifiers to ensure they are non-empty and ASCII-safe.
pub(crate) fn validate_identifier(value: &str, label: &str) -> GResult<()> {
    if value.is_empty() {
        return Err(GreenticError::new(
            ErrorCode::InvalidInput,
            format!("{label} must not be empty"),
        ));
    }
    if value
        .chars()
        .any(|c| !(c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-')))
    {
        return Err(GreenticError::new(
            ErrorCode::InvalidInput,
            format!("{label} must contain only ASCII letters, digits, '.', '-', or '_'"),
        ));
    }
    Ok(())
}

/// Validates API key references that may include URI-like prefixes.
pub(crate) fn validate_api_key_ref(value: &str) -> GResult<()> {
    if value.trim().is_empty() {
        return Err(GreenticError::new(
            ErrorCode::InvalidInput,
            "ApiKeyRef must not be empty",
        ));
    }
    if value.chars().any(char::is_whitespace) {
        return Err(GreenticError::new(
            ErrorCode::InvalidInput,
            "ApiKeyRef must not contain whitespace",
        ));
    }
    if !value.is_ascii() {
        return Err(GreenticError::new(
            ErrorCode::InvalidInput,
            "ApiKeyRef must contain only ASCII characters",
        ));
    }
    Ok(())
}

/// Canonical schema IDs for the exported document types.
pub mod ids {
    /// Pack identifier schema.
    pub const PACK_ID: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/pack-id.schema.json";
    /// Component identifier schema.
    pub const COMPONENT_ID: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/component-id.schema.json";
    /// Flow identifier schema.
    pub const FLOW_ID: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/flow-id.schema.json";
    /// Node identifier schema.
    pub const NODE_ID: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/node-id.schema.json";
    /// Tenant context schema.
    pub const TENANT_CONTEXT: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/tenant-context.schema.json";
    /// Hash digest schema.
    pub const HASH_DIGEST: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/hash-digest.schema.json";
    /// Semantic version requirement schema.
    pub const SEMVER_REQ: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/semver-req.schema.json";
    /// Redaction path schema.
    pub const REDACTION_PATH: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/redaction-path.schema.json";
    /// Capabilities schema.
    pub const CAPABILITIES: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/capabilities.schema.json";
    /// RepoSkin (skin.json) schema.
    pub const REPO_SKIN: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/repo-skin.schema.json";
    /// RepoAuth (auth.json) schema.
    pub const REPO_AUTH: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/repo-auth.schema.json";
    /// RepoTenantConfig (config.json) schema.
    pub const REPO_TENANT_CONFIG: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/repo-tenant-config.schema.json";
    /// Tenant DID document (did.json) schema.
    pub const TENANT_DID_DOCUMENT: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/tenant-did-document.schema.json";
    /// Flow schema.
    pub const FLOW: &str = "greentic.flow.v1";
    /// Node schema.
    pub const NODE: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/node.schema.json";
    /// Component manifest schema.
    pub const COMPONENT_MANIFEST: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/component-manifest.schema.json";
    /// Pack manifest schema.
    pub const PACK_MANIFEST: &str = "greentic.pack-manifest.v1";
    /// Limits schema.
    pub const LIMITS: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/limits.schema.json";
    /// Telemetry spec schema.
    pub const TELEMETRY_SPEC: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/telemetry-spec.schema.json";
    /// Node summary schema.
    pub const NODE_SUMMARY: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/node-summary.schema.json";
    /// Node failure schema.
    pub const NODE_FAILURE: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/node-failure.schema.json";
    /// Node status schema.
    pub const NODE_STATUS: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/node-status.schema.json";
    /// Run status schema.
    pub const RUN_STATUS: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/run-status.schema.json";
    /// Transcript offset schema.
    pub const TRANSCRIPT_OFFSET: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/transcript-offset.schema.json";
    /// Tools capability schema.
    pub const TOOLS_CAPS: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/tools-caps.schema.json";
    /// Secrets capability schema.
    pub const SECRETS_CAPS: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/secrets-caps.schema.json";
    /// Branch reference schema.
    pub const BRANCH_REF: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/branch-ref.schema.json";
    /// Commit reference schema.
    pub const COMMIT_REF: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/commit-ref.schema.json";
    /// Git provider reference schema.
    pub const GIT_PROVIDER_REF: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/git-provider-ref.schema.json";
    /// Scanner provider reference schema.
    pub const SCANNER_REF: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/scanner-ref.schema.json";
    /// Webhook identifier schema.
    pub const WEBHOOK_ID: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/webhook-id.schema.json";
    /// Repository reference schema.
    pub const REPO_REF: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/repo-ref.schema.json";
    /// Component reference schema.
    pub const COMPONENT_REF: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/component-ref.schema.json";
    /// Version reference schema.
    pub const VERSION_REF: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/version-ref.schema.json";
    /// Build reference schema.
    pub const BUILD_REF: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/build-ref.schema.json";
    /// Scan reference schema.
    pub const SCAN_REF: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/scan-ref.schema.json";
    /// Attestation reference schema.
    pub const ATTESTATION_REF: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/attestation-ref.schema.json";
    /// Attestation id schema.
    pub const ATTESTATION_ID: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/attestation-id.schema.json";
    /// Policy reference schema.
    pub const POLICY_REF: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/policy-ref.schema.json";
    /// Policy input reference schema.
    pub const POLICY_INPUT_REF: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/policy-input-ref.schema.json";
    /// Store reference schema.
    pub const STORE_REF: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/store-ref.schema.json";
    /// Registry reference schema.
    pub const REGISTRY_REF: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/registry-ref.schema.json";
    /// OCI image reference schema.
    pub const OCI_IMAGE_REF: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/oci-image-ref.schema.json";
    /// Artifact reference schema.
    pub const ARTIFACT_REF: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/artifact-ref.schema.json";
    /// SBOM reference schema.
    pub const SBOM_REF: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/sbom-ref.schema.json";
    /// Signing key reference schema.
    pub const SIGNING_KEY_REF: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/signing-key-ref.schema.json";
    /// Signature reference schema.
    pub const SIGNATURE_REF: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/signature-ref.schema.json";
    /// Statement reference schema.
    pub const STATEMENT_REF: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/statement-ref.schema.json";
    /// Build log reference schema.
    pub const BUILD_LOG_REF: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/build-log-ref.schema.json";
    /// Metadata record reference schema.
    pub const METADATA_RECORD_REF: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/metadata-record-ref.schema.json";
    /// API key reference schema.
    pub const API_KEY_REF: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/api-key-ref.schema.json";
    /// Environment reference schema.
    pub const ENVIRONMENT_REF: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/environment-ref.schema.json";
    /// Distributor reference schema.
    pub const DISTRIBUTOR_REF: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/distributor-ref.schema.json";
    /// Storefront identifier schema.
    pub const STOREFRONT_ID: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/storefront-id.schema.json";
    /// Store product identifier schema.
    pub const STORE_PRODUCT_ID: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/store-product-id.schema.json";
    /// Store plan identifier schema.
    pub const STORE_PLAN_ID: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/store-plan-id.schema.json";
    /// Subscription identifier schema.
    pub const SUBSCRIPTION_ID: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/subscription-id.schema.json";
    /// Bundle identifier schema.
    pub const BUNDLE_ID: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/bundle-id.schema.json";
    /// Collection identifier schema.
    pub const COLLECTION_ID: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/collection-id.schema.json";
    /// Artifact selector schema.
    pub const ARTIFACT_SELECTOR: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/artifact-selector.schema.json";
    /// Capability map schema.
    pub const CAPABILITY_MAP: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/capability-map.schema.json";
    /// Store product kind schema.
    pub const STORE_PRODUCT_KIND: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/store-product-kind.schema.json";
    /// Version strategy schema.
    pub const VERSION_STRATEGY: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/version-strategy.schema.json";
    /// Rollout status schema.
    pub const ROLLOUT_STATUS: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/rollout-status.schema.json";
    /// Connection kind schema.
    pub const CONNECTION_KIND: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/connection-kind.schema.json";
    /// Pack or component reference schema.
    pub const PACK_OR_COMPONENT_REF: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/pack-or-component-ref.schema.json";
    /// Plan limits schema.
    pub const PLAN_LIMITS: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/plan-limits.schema.json";
    /// Price model schema.
    pub const PRICE_MODEL: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/price-model.schema.json";
    /// Subscription status schema.
    pub const SUBSCRIPTION_STATUS: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/subscription-status.schema.json";
    /// Build plan schema.
    pub const BUILD_PLAN: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/build-plan.schema.json";
    /// Build status schema.
    pub const BUILD_STATUS: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/build-status.schema.json";
    /// Scan request schema.
    pub const SCAN_REQUEST: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/scan-request.schema.json";
    /// Scan result schema.
    pub const SCAN_RESULT: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/scan-result.schema.json";
    /// Sign request schema.
    pub const SIGN_REQUEST: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/sign-request.schema.json";
    /// Verify request schema.
    pub const VERIFY_REQUEST: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/verify-request.schema.json";
    /// Verify result schema.
    pub const VERIFY_RESULT: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/verify-result.schema.json";
    /// Attestation statement schema.
    pub const ATTESTATION_STATEMENT: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/attestation-statement.schema.json";
    /// Metadata record schema.
    pub const METADATA_RECORD: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/metadata-record.schema.json";
    /// Repository context schema.
    pub const REPO_CONTEXT: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/repo-context.schema.json";
    /// Store context schema.
    pub const STORE_CONTEXT: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/store-context.schema.json";
    /// Bundle schema.
    pub const BUNDLE: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/bundle.schema.json";
    /// Bundle export specification schema.
    pub const DESIRED_STATE_EXPORT: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/desired-state-export.schema.json";
    /// Desired state schema.
    pub const DESIRED_STATE: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/desired-state.schema.json";
    /// Desired subscription entry schema.
    pub const DESIRED_SUBSCRIPTION_ENTRY: &str = "https://greentic-ai.github.io/greentic-types/schemas/v1/desired-subscription-entry.schema.json";
    /// Storefront schema.
    pub const STOREFRONT: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/storefront.schema.json";
    /// Store product schema.
    pub const STORE_PRODUCT: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/store-product.schema.json";
    /// Store plan schema.
    pub const STORE_PLAN: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/store-plan.schema.json";
    /// Subscription schema.
    pub const SUBSCRIPTION: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/subscription.schema.json";
    /// Environment schema.
    pub const ENVIRONMENT: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/environment.schema.json";
    /// Store theme schema.
    pub const THEME: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/theme.schema.json";
    /// Layout section schema.
    pub const LAYOUT_SECTION: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/layout-section.schema.json";
    /// Collection schema.
    pub const COLLECTION: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/collection.schema.json";
    /// Product override schema.
    pub const PRODUCT_OVERRIDE: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/product-override.schema.json";
    /// Event envelope schema.
    pub const EVENT_ENVELOPE: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/event-envelope.schema.json";
    /// Event provider descriptor schema.
    pub const EVENT_PROVIDER_DESCRIPTOR: &str = "https://greentic-ai.github.io/greentic-types/schemas/v1/event-provider-descriptor.schema.json";
    /// Channel message envelope schema.
    pub const CHANNEL_MESSAGE_ENVELOPE: &str = "https://greentic-ai.github.io/greentic-types/schemas/v1/channel-message-envelope.schema.json";
    /// Attachment schema.
    pub const ATTACHMENT: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/attachment.schema.json";
    /// Worker request envelope schema.
    pub const WORKER_REQUEST: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/worker-request.schema.json";
    /// Worker message schema.
    pub const WORKER_MESSAGE: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/worker-message.schema.json";
    /// Worker response envelope schema.
    pub const WORKER_RESPONSE: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/worker-response.schema.json";
    /// OTLP attribute key schema.
    pub const OTLP_KEYS: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/otlp-keys.schema.json";
    /// Run result schema.
    pub const RUN_RESULT: &str =
        "https://greentic-ai.github.io/greentic-types/schemas/v1/run-result.schema.json";
}

#[cfg(all(feature = "schema", feature = "std"))]
/// Writes every JSON Schema to the provided directory.
pub fn write_all_schemas(out_dir: &std::path::Path) -> anyhow::Result<()> {
    use anyhow::Context;
    use std::fs;

    fs::create_dir_all(out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;

    for entry in crate::schema::entries() {
        let schema = (entry.generator)();
        let path = out_dir.join(entry.file_name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        let json =
            serde_json::to_vec_pretty(&schema).context("failed to serialize schema to JSON")?;
        fs::write(&path, json).with_context(|| format!("failed to write {}", path.display()))?;
    }

    Ok(())
}

macro_rules! id_newtype {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
        #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
        #[cfg_attr(feature = "schemars", derive(JsonSchema))]
        #[cfg_attr(feature = "serde", serde(try_from = "String", into = "String"))]
        pub struct $name(pub String);

        impl $name {
            /// Returns the identifier as a string slice.
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Validates and constructs the identifier from the provided value.
            pub fn new(value: impl AsRef<str>) -> GResult<Self> {
                value.as_ref().parse()
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl FromStr for $name {
            type Err = GreenticError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                validate_identifier(value, stringify!($name))?;
                Ok(Self(value.to_owned()))
            }
        }

        impl TryFrom<String> for $name {
            type Error = GreenticError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                $name::from_str(&value)
            }
        }

        impl TryFrom<&str> for $name {
            type Error = GreenticError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                $name::from_str(value)
            }
        }
    };
}

id_newtype!(EnvId, "Environment identifier for a tenant context.");
id_newtype!(TenantId, "Tenant identifier within an environment.");
id_newtype!(TeamId, "Team identifier belonging to a tenant.");
id_newtype!(UserId, "User identifier within a tenant.");
id_newtype!(BranchRef, "Reference to a source control branch.");
id_newtype!(CommitRef, "Reference to a source control commit.");
id_newtype!(
    GitProviderRef,
    "Identifier referencing a source control provider."
);
id_newtype!(ScannerRef, "Identifier referencing a scanner provider.");
id_newtype!(WebhookId, "Identifier referencing a registered webhook.");
id_newtype!(PackId, "Globally unique pack identifier.");
id_newtype!(
    ComponentId,
    "Identifier referencing a component binding in a pack."
);
id_newtype!(FlowId, "Identifier referencing a flow inside a pack.");
id_newtype!(NodeId, "Identifier referencing a node inside a flow graph.");
id_newtype!(
    EnvironmentRef,
    "Identifier referencing a deployment environment."
);
id_newtype!(
    DistributorRef,
    "Identifier referencing a distributor instance."
);
id_newtype!(StoreFrontId, "Identifier referencing a storefront.");
id_newtype!(
    StoreProductId,
    "Identifier referencing a product in the store catalog."
);
id_newtype!(
    StorePlanId,
    "Identifier referencing a plan for a store product."
);
id_newtype!(
    SubscriptionId,
    "Identifier referencing a subscription entry."
);
id_newtype!(BundleId, "Identifier referencing a distributor bundle.");
id_newtype!(CollectionId, "Identifier referencing a product collection.");
id_newtype!(RepoRef, "Repository reference within a supply chain.");
id_newtype!(
    ComponentRef,
    "Supply-chain component reference (distinct from pack ComponentId)."
);
id_newtype!(
    VersionRef,
    "Version reference for a component or metadata record."
);
id_newtype!(BuildRef, "Build reference within a supply chain.");
id_newtype!(ScanRef, "Scan reference within a supply chain.");
id_newtype!(
    AttestationRef,
    "Attestation reference within a supply chain."
);
id_newtype!(AttestationId, "Identifier referencing an attestation.");
id_newtype!(PolicyRef, "Policy reference within a supply chain.");
id_newtype!(
    PolicyInputRef,
    "Reference to a policy input payload for evaluation."
);
id_newtype!(StoreRef, "Content store reference within a supply chain.");
id_newtype!(
    RegistryRef,
    "Registry reference for OCI or artifact storage."
);
id_newtype!(OciImageRef, "Reference to an OCI image for distribution.");
id_newtype!(
    ArtifactRef,
    "Artifact reference within a build or scan result."
);
id_newtype!(
    SbomRef,
    "Reference to a Software Bill of Materials artifact."
);
id_newtype!(SigningKeyRef, "Reference to a signing key handle.");
id_newtype!(SignatureRef, "Reference to a generated signature.");
id_newtype!(StatementRef, "Reference to an attestation statement.");
id_newtype!(
    BuildLogRef,
    "Reference to a build log output produced during execution."
);
id_newtype!(
    MetadataRecordRef,
    "Reference to a metadata record attached to artifacts or bundles."
);

/// API key reference used across secrets providers without exposing key material.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(try_from = "String", into = "String"))]
pub struct ApiKeyRef(pub String);

impl ApiKeyRef {
    /// Returns the reference as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validates and constructs the reference from the provided value.
    pub fn new(value: impl AsRef<str>) -> GResult<Self> {
        value.as_ref().parse()
    }
}

impl fmt::Display for ApiKeyRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ApiKeyRef {
    type Err = GreenticError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        validate_api_key_ref(value)?;
        Ok(Self(value.to_owned()))
    }
}

impl TryFrom<String> for ApiKeyRef {
    type Error = GreenticError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        ApiKeyRef::from_str(&value)
    }
}

impl TryFrom<&str> for ApiKeyRef {
    type Error = GreenticError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        ApiKeyRef::from_str(value)
    }
}

impl From<ApiKeyRef> for String {
    fn from(value: ApiKeyRef) -> Self {
        value.0
    }
}

impl AsRef<str> for ApiKeyRef {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Compact tenant summary propagated to developers and tooling.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct TenantContext {
    /// Tenant identifier owning the execution.
    pub tenant_id: TenantId,
    /// Optional team identifier scoped to the tenant.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub team_id: Option<TeamId>,
    /// Optional user identifier scoped to the tenant.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub user_id: Option<UserId>,
    /// Optional session identifier for end-to-end correlation.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub session_id: Option<String>,
    /// Optional attributes for routing and tracing.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "BTreeMap::is_empty")
    )]
    pub attributes: BTreeMap<String, String>,
}

impl TenantContext {
    /// Creates a new tenant context scoped to the provided tenant id.
    pub fn new(tenant_id: TenantId) -> Self {
        Self {
            tenant_id,
            team_id: None,
            user_id: None,
            session_id: None,
            attributes: BTreeMap::new(),
        }
    }
}

impl From<&TenantCtx> for TenantContext {
    fn from(ctx: &TenantCtx) -> Self {
        Self {
            tenant_id: ctx.tenant_id.clone(),
            team_id: ctx.team_id.clone().or_else(|| ctx.team.clone()),
            user_id: ctx.user_id.clone().or_else(|| ctx.user.clone()),
            session_id: ctx.session_id.clone(),
            attributes: ctx.attributes.clone(),
        }
    }
}

/// Supported hashing algorithms for pack content digests.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum HashAlgorithm {
    /// Blake3 hashing algorithm.
    Blake3,
    /// Catch all for other algorithms.
    Other(String),
}

/// Content digest describing a pack or artifact.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(into = "HashDigestRepr", try_from = "HashDigestRepr")
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct HashDigest {
    /// Hash algorithm used to produce the digest.
    pub algo: HashAlgorithm,
    /// Hex encoded digest bytes.
    pub hex: String,
}

impl HashDigest {
    /// Creates a new digest ensuring the hex payload is valid.
    pub fn new(algo: HashAlgorithm, hex: impl Into<String>) -> GResult<Self> {
        let hex = hex.into();
        validate_hex(&hex)?;
        Ok(Self { algo, hex })
    }

    /// Convenience constructor for Blake3 digests.
    pub fn blake3(hex: impl Into<String>) -> GResult<Self> {
        Self::new(HashAlgorithm::Blake3, hex)
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
struct HashDigestRepr {
    algo: HashAlgorithm,
    hex: String,
}

impl From<HashDigest> for HashDigestRepr {
    fn from(value: HashDigest) -> Self {
        Self {
            algo: value.algo,
            hex: value.hex,
        }
    }
}

impl TryFrom<HashDigestRepr> for HashDigest {
    type Error = GreenticError;

    fn try_from(value: HashDigestRepr) -> Result<Self, Self::Error> {
        HashDigest::new(value.algo, value.hex)
    }
}

fn validate_hex(hex: &str) -> GResult<()> {
    if hex.is_empty() {
        return Err(GreenticError::new(
            ErrorCode::InvalidInput,
            "digest hex payload must not be empty",
        ));
    }
    if hex.len() % 2 != 0 {
        return Err(GreenticError::new(
            ErrorCode::InvalidInput,
            "digest hex payload must have an even number of digits",
        ));
    }
    if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(GreenticError::new(
            ErrorCode::InvalidInput,
            "digest hex payload must be hexadecimal",
        ));
    }
    Ok(())
}

/// Semantic version requirement validated by [`semver`].
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(into = "String", try_from = "String"))]
pub struct SemverReq(String);

impl SemverReq {
    /// Parses and validates a semantic version requirement string.
    pub fn parse(value: impl AsRef<str>) -> GResult<Self> {
        let value = value.as_ref();
        VersionReq::parse(value).map_err(|err| {
            GreenticError::new(
                ErrorCode::InvalidInput,
                format!("invalid semver requirement '{value}': {err}"),
            )
        })?;
        Ok(Self(value.to_owned()))
    }

    /// Returns the underlying string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Converts into a [`semver::VersionReq`].
    pub fn to_version_req(&self) -> VersionReq {
        VersionReq::parse(&self.0)
            .unwrap_or_else(|err| unreachable!("SemverReq::parse validated inputs: {err}"))
    }
}

impl fmt::Display for SemverReq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<SemverReq> for String {
    fn from(value: SemverReq) -> Self {
        value.0
    }
}

impl TryFrom<String> for SemverReq {
    type Error = GreenticError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        SemverReq::parse(&value)
    }
}

impl TryFrom<&str> for SemverReq {
    type Error = GreenticError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        SemverReq::parse(value)
    }
}

impl FromStr for SemverReq {
    type Err = GreenticError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        SemverReq::parse(s)
    }
}

/// JSONPath expression pointing at sensitive fields that should be redacted.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(into = "String", try_from = "String"))]
pub struct RedactionPath(String);

impl RedactionPath {
    /// Validates and stores a JSONPath expression.
    pub fn parse(value: impl AsRef<str>) -> GResult<Self> {
        let value = value.as_ref();
        validate_jsonpath(value)?;
        Ok(Self(value.to_owned()))
    }

    /// Returns the JSONPath string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RedactionPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<RedactionPath> for String {
    fn from(value: RedactionPath) -> Self {
        value.0
    }
}

impl TryFrom<String> for RedactionPath {
    type Error = GreenticError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        RedactionPath::parse(&value)
    }
}

impl TryFrom<&str> for RedactionPath {
    type Error = GreenticError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        RedactionPath::parse(value)
    }
}

fn validate_jsonpath(path: &str) -> GResult<()> {
    if path.is_empty() {
        return Err(GreenticError::new(
            ErrorCode::InvalidInput,
            "redaction path cannot be empty",
        ));
    }
    if !path.starts_with('$') {
        return Err(GreenticError::new(
            ErrorCode::InvalidInput,
            "redaction path must start with '$'",
        ));
    }
    if path.chars().any(|c| c.is_control()) {
        return Err(GreenticError::new(
            ErrorCode::InvalidInput,
            "redaction path cannot contain control characters",
        ));
    }
    Ok(())
}

#[cfg(feature = "schemars")]
impl JsonSchema for SemverReq {
    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("SemverReq")
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let mut schema = <String>::json_schema(generator);
        if schema.get("description").is_none() {
            schema.insert(
                "description".into(),
                "Validated semantic version requirement string".into(),
            );
        }
        schema
    }
}

#[cfg(feature = "schemars")]
impl JsonSchema for RedactionPath {
    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("RedactionPath")
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let mut schema = <String>::json_schema(generator);
        if schema.get("description").is_none() {
            schema.insert(
                "description".into(),
                "JSONPath expression used for runtime redaction".into(),
            );
        }
        schema
    }
}

/// Deadline metadata for an invocation, stored as Unix epoch milliseconds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct InvocationDeadline {
    unix_millis: i128,
}

impl InvocationDeadline {
    /// Creates a deadline from a Unix timestamp expressed in milliseconds.
    pub const fn from_unix_millis(unix_millis: i128) -> Self {
        Self { unix_millis }
    }

    /// Returns the deadline as Unix epoch milliseconds.
    pub const fn unix_millis(&self) -> i128 {
        self.unix_millis
    }

    /// Converts the deadline into an [`OffsetDateTime`].
    #[cfg(feature = "time")]
    pub fn to_offset_date_time(&self) -> Result<OffsetDateTime, time::error::ComponentRange> {
        OffsetDateTime::from_unix_timestamp_nanos(self.unix_millis * 1_000_000)
    }

    /// Creates a deadline from an [`OffsetDateTime`], truncating to milliseconds.
    #[cfg(feature = "time")]
    pub fn from_offset_date_time(value: OffsetDateTime) -> Self {
        let nanos = value.unix_timestamp_nanos();
        Self {
            unix_millis: nanos / 1_000_000,
        }
    }
}

/// Context that accompanies every invocation across Greentic runtimes.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct TenantCtx {
    /// Environment scope (for example `dev`, `staging`, or `prod`).
    pub env: EnvId,
    /// Tenant identifier for the current execution.
    pub tenant: TenantId,
    /// Stable tenant identifier reference used across systems.
    pub tenant_id: TenantId,
    /// Optional team identifier scoped to the tenant.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub team: Option<TeamId>,
    /// Optional team identifier accessible via the shared schema.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub team_id: Option<TeamId>,
    /// Optional user identifier scoped to the tenant.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub user: Option<UserId>,
    /// Optional user identifier aligned with the shared schema.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub user_id: Option<UserId>,
    /// Optional session identifier propagated by the runtime.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub session_id: Option<String>,
    /// Optional flow identifier for the current execution.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub flow_id: Option<String>,
    /// Optional node identifier within the flow.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub node_id: Option<String>,
    /// Optional provider identifier describing the runtime surface.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub provider_id: Option<String>,
    /// Distributed tracing identifier when available.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub trace_id: Option<String>,
    /// Correlation identifier for linking related events.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub correlation_id: Option<String>,
    /// Free-form attributes for routing and tracing.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "BTreeMap::is_empty")
    )]
    pub attributes: BTreeMap<String, String>,
    /// Deadline when the invocation should finish.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub deadline: Option<InvocationDeadline>,
    /// Attempt counter for retried invocations (starting at zero).
    pub attempt: u32,
    /// Stable idempotency key propagated across retries.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub idempotency_key: Option<String>,
    /// Optional impersonation context describing the acting identity.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub impersonation: Option<Impersonation>,
}

impl TenantCtx {
    /// Creates a new tenant context with the provided environment and tenant identifiers.
    pub fn new(env: EnvId, tenant: TenantId) -> Self {
        let tenant_id = tenant.clone();
        Self {
            env,
            tenant: tenant.clone(),
            tenant_id,
            team: None,
            team_id: None,
            user: None,
            user_id: None,
            session_id: None,
            flow_id: None,
            node_id: None,
            provider_id: None,
            trace_id: None,
            correlation_id: None,
            attributes: BTreeMap::new(),
            deadline: None,
            attempt: 0,
            idempotency_key: None,
            impersonation: None,
        }
    }

    /// Updates the team information ensuring legacy and shared fields stay aligned.
    pub fn with_team(mut self, team: Option<TeamId>) -> Self {
        self.team = team.clone();
        self.team_id = team;
        self
    }

    /// Updates the user information ensuring legacy and shared fields stay aligned.
    pub fn with_user(mut self, user: Option<UserId>) -> Self {
        self.user = user.clone();
        self.user_id = user;
        self
    }

    /// Updates the session identifier.
    pub fn with_session(mut self, session: impl Into<String>) -> Self {
        self.session_id = Some(session.into());
        self
    }

    /// Updates the flow identifier.
    pub fn with_flow(mut self, flow: impl Into<String>) -> Self {
        self.flow_id = Some(flow.into());
        self
    }

    /// Updates the node identifier.
    pub fn with_node(mut self, node: impl Into<String>) -> Self {
        self.node_id = Some(node.into());
        self
    }

    /// Updates the provider identifier.
    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider_id = Some(provider.into());
        self
    }

    /// Attaches or replaces the attributes map.
    pub fn with_attributes(mut self, attributes: BTreeMap<String, String>) -> Self {
        self.attributes = attributes;
        self
    }

    /// Sets the impersonation context.
    pub fn with_impersonation(mut self, impersonation: Option<Impersonation>) -> Self {
        self.impersonation = impersonation;
        self
    }

    /// Returns a copy of the context with the provided attempt value.
    pub fn with_attempt(mut self, attempt: u32) -> Self {
        self.attempt = attempt;
        self
    }

    /// Updates the deadline metadata for subsequent invocations.
    pub fn with_deadline(mut self, deadline: Option<InvocationDeadline>) -> Self {
        self.deadline = deadline;
        self
    }

    /// Returns the session identifier, when present.
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Returns the flow identifier, when present.
    pub fn flow_id(&self) -> Option<&str> {
        self.flow_id.as_deref()
    }

    /// Returns the node identifier, when present.
    pub fn node_id(&self) -> Option<&str> {
        self.node_id.as_deref()
    }

    /// Returns the provider identifier, when present.
    pub fn provider_id(&self) -> Option<&str> {
        self.provider_id.as_deref()
    }
}

/// Primary payload representation shared across envelopes.
pub type BinaryPayload = Vec<u8>;

/// Normalized ingress payload delivered to nodes.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct InvocationEnvelope {
    /// Tenant context for the invocation.
    pub ctx: TenantCtx,
    /// Flow identifier the event belongs to.
    pub flow_id: String,
    /// Optional node identifier within the flow.
    pub node_id: Option<String>,
    /// Operation being invoked (for example `on_message` or `tick`).
    pub op: String,
    /// Normalized payload for the invocation.
    pub payload: BinaryPayload,
    /// Raw metadata propagated from the ingress surface.
    pub metadata: BinaryPayload,
}

/// Structured detail payload attached to a node error.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum ErrorDetail {
    /// UTF-8 encoded detail payload.
    Text(String),
    /// Binary payload detail (for example message pack or CBOR).
    Binary(BinaryPayload),
}

/// Error type emitted by Greentic nodes.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct NodeError {
    /// Machine readable error code.
    pub code: String,
    /// Human readable message explaining the failure.
    pub message: String,
    /// Whether the failure is retryable by the runtime.
    pub retryable: bool,
    /// Optional backoff duration in milliseconds for the next retry.
    pub backoff_ms: Option<u64>,
    /// Optional structured error detail payload.
    pub details: Option<ErrorDetail>,
    #[cfg(feature = "std")]
    #[cfg_attr(feature = "serde", serde(skip, default = "default_source"))]
    #[cfg_attr(feature = "schemars", schemars(skip))]
    source: Option<Box<dyn StdError + Send + Sync>>,
}

#[cfg(all(feature = "std", feature = "serde"))]
fn default_source() -> Option<Box<dyn StdError + Send + Sync>> {
    None
}

impl NodeError {
    /// Constructs a non-retryable failure with the supplied code and message.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            retryable: false,
            backoff_ms: None,
            details: None,
            #[cfg(feature = "std")]
            source: None,
        }
    }

    /// Marks the error as retryable with an optional backoff value.
    pub fn with_retry(mut self, backoff_ms: Option<u64>) -> Self {
        self.retryable = true;
        self.backoff_ms = backoff_ms;
        self
    }

    /// Attaches structured details to the error.
    pub fn with_detail(mut self, detail: ErrorDetail) -> Self {
        self.details = Some(detail);
        self
    }

    /// Attaches a textual detail payload to the error.
    pub fn with_detail_text(mut self, detail: impl Into<String>) -> Self {
        self.details = Some(ErrorDetail::Text(detail.into()));
        self
    }

    /// Attaches a binary detail payload to the error.
    pub fn with_detail_binary(mut self, detail: BinaryPayload) -> Self {
        self.details = Some(ErrorDetail::Binary(detail));
        self
    }

    #[cfg(feature = "std")]
    /// Attaches a source error to the failure for debugging purposes.
    pub fn with_source<E>(mut self, source: E) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        self.source = Some(Box::new(source));
        self
    }

    /// Returns the structured details, when available.
    pub fn detail(&self) -> Option<&ErrorDetail> {
        self.details.as_ref()
    }

    #[cfg(feature = "std")]
    /// Returns the attached source error when one has been provided.
    pub fn source(&self) -> Option<&(dyn StdError + Send + Sync + 'static)> {
        self.source.as_deref()
    }
}

impl fmt::Display for NodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

#[cfg(feature = "std")]
impl StdError for NodeError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source
            .as_ref()
            .map(|err| err.as_ref() as &(dyn StdError + 'static))
    }
}

/// Alias for results returned by node handlers.
pub type NodeResult<T> = Result<T, NodeError>;

/// Generates a stable idempotency key for a node invocation.
///
/// The key uses tenant, flow, node, and correlation identifiers. Missing
/// correlation values fall back to the value stored on the context.
pub fn make_idempotency_key(
    ctx: &TenantCtx,
    flow_id: &str,
    node_id: Option<&str>,
    correlation: Option<&str>,
) -> String {
    let node_segment = node_id.unwrap_or_default();
    let correlation_segment = correlation
        .or(ctx.correlation_id.as_deref())
        .unwrap_or_default();
    let input = format!(
        "{}|{}|{}|{}",
        ctx.tenant_id.as_str(),
        flow_id,
        node_segment,
        correlation_segment
    );
    fnv1a_128_hex(input.as_bytes())
}

const FNV_OFFSET_BASIS: u128 = 0x6c62272e07bb014262b821756295c58d;
const FNV_PRIME: u128 = 0x0000000001000000000000000000013b;

fn fnv1a_128_hex(bytes: &[u8]) -> String {
    let mut hash = FNV_OFFSET_BASIS;
    for &byte in bytes {
        hash ^= byte as u128;
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    let mut output = String::with_capacity(32);
    for shift in (0..32).rev() {
        let nibble = ((hash >> (shift * 4)) & 0x0f) as u8;
        output.push(match nibble {
            0..=9 => (b'0' + nibble) as char,
            _ => (b'a' + (nibble - 10)) as char,
        });
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::convert::TryFrom;
    use time::OffsetDateTime;

    fn sample_ctx() -> TenantCtx {
        let env = EnvId::try_from("prod").unwrap_or_else(|err| panic!("{err}"));
        let tenant = TenantId::try_from("tenant-123").unwrap_or_else(|err| panic!("{err}"));
        let team = TeamId::try_from("team-456").unwrap_or_else(|err| panic!("{err}"));
        let user = UserId::try_from("user-789").unwrap_or_else(|err| panic!("{err}"));

        let mut ctx = TenantCtx::new(env, tenant)
            .with_team(Some(team))
            .with_user(Some(user))
            .with_attempt(2)
            .with_deadline(Some(InvocationDeadline::from_unix_millis(
                1_700_000_000_000,
            )));
        ctx.trace_id = Some("trace-abc".to_owned());
        ctx.correlation_id = Some("corr-xyz".to_owned());
        ctx.idempotency_key = Some("key-123".to_owned());
        ctx
    }

    #[test]
    fn idempotent_key_stable() {
        let ctx = sample_ctx();
        let key_a = make_idempotency_key(&ctx, "flow-1", Some("node-1"), Some("corr-override"));
        let key_b = make_idempotency_key(&ctx, "flow-1", Some("node-1"), Some("corr-override"));
        assert_eq!(key_a, key_b);
        assert_eq!(key_a.len(), 32);
    }

    #[test]
    fn idempotent_key_uses_context_correlation() {
        let ctx = sample_ctx();
        let key = make_idempotency_key(&ctx, "flow-1", None, None);
        let expected = make_idempotency_key(&ctx, "flow-1", None, ctx.correlation_id.as_deref());
        assert_eq!(key, expected);
    }

    #[test]
    #[cfg(feature = "time")]
    fn deadline_roundtrips_through_offset_datetime() {
        let dt = OffsetDateTime::from_unix_timestamp(1_700_000_000)
            .unwrap_or_else(|err| panic!("valid timestamp: {err}"));
        let deadline = InvocationDeadline::from_offset_date_time(dt);
        let roundtrip = deadline
            .to_offset_date_time()
            .unwrap_or_else(|err| panic!("round-trip conversion failed: {err}"));
        let millis = dt.unix_timestamp_nanos() / 1_000_000;
        assert_eq!(deadline.unix_millis(), millis);
        assert_eq!(roundtrip.unix_timestamp_nanos() / 1_000_000, millis);
    }

    #[test]
    fn node_error_builder_sets_fields() {
        let err = NodeError::new("TEST", "example")
            .with_retry(Some(500))
            .with_detail_text("context");

        assert!(err.retryable);
        assert_eq!(err.backoff_ms, Some(500));
        match err.detail() {
            Some(ErrorDetail::Text(detail)) => assert_eq!(detail, "context"),
            other => panic!("unexpected detail {other:?}"),
        }
    }

    #[cfg(feature = "std")]
    #[test]
    fn node_error_source_roundtrips() {
        use std::io::Error;

        let source = Error::other("boom");
        let err = NodeError::new("TEST", "example").with_source(source);
        assert!(err.source().is_some());
    }
}
