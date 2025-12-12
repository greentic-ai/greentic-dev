#![forbid(unsafe_code)]

#[cfg(feature = "abi")]
pub mod abi;
pub mod capabilities;
#[cfg(feature = "cli")]
pub mod config;
#[cfg(feature = "describe")]
pub mod describe;
pub mod error;
pub mod lifecycle;
pub mod limits;
#[cfg(feature = "loader")]
pub mod loader;
pub mod manifest;
pub mod path_safety;
#[cfg(feature = "prepare")]
pub mod prepare;
pub mod provenance;
pub mod schema;
pub mod security;
pub mod signing;
pub mod telemetry;

pub mod store;

#[cfg(feature = "cli")]
pub mod cli;
#[cfg(feature = "cli")]
pub mod cmd;
#[cfg(feature = "cli")]
pub mod scaffold;
#[cfg(any(
    feature = "abi",
    feature = "describe",
    feature = "prepare",
    feature = "cli"
))]
pub mod wasm;

#[cfg(feature = "abi")]
pub use abi::{AbiError, check_world, has_lifecycle};
pub use capabilities::{Capabilities, CapabilityError};
#[cfg(feature = "describe")]
pub use describe::{
    DescribeError, DescribePayload, DescribeVersion, from_embedded, from_exported_func,
    from_wit_world, load as load_describe,
};
pub use error::ComponentError;
pub use lifecycle::Lifecycle;
pub use limits::{LimitError, LimitOverrides, Limits, defaults_dev, merge};
#[cfg(feature = "loader")]
pub use loader::{ComponentHandle, LoadError, discover};
pub use manifest::{
    Artifacts, ComponentManifest, DescribeExport, DescribeKind, Hashes, ManifestError, ManifestId,
    WasmHash, World, parse_manifest, schema as manifest_schema, validate_manifest,
};
#[cfg(feature = "prepare")]
pub use prepare::{PackEntry, PreparedComponent, RunnerConfig, clear_cache_for, prepare_component};
pub use provenance::{Provenance, ProvenanceError};
pub use schema::{
    JsonPath, collect_capability_hints, collect_default_annotations, collect_redactions,
};
pub type RedactionPath = JsonPath;
pub use security::{Profile, enforce_capabilities};
pub use signing::{
    DevPolicy, SignatureRef, SigningError, StrictPolicy, compute_wasm_hash, verify_manifest_hash,
    verify_wasm_hash,
};
pub use store::{
    CompatError, CompatPolicy, ComponentBytes, ComponentId, ComponentLocator, ComponentStore,
    MetaInfo, SourceId,
};
pub use telemetry::{TelemetrySpec, span_name};
