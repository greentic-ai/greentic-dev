//! Telemetry helpers exposed by `greentic-types`.

#[cfg(feature = "otel-keys")]
mod keys;
mod span_context;

#[cfg(feature = "otel-keys")]
pub use keys::OtlpKeys;
pub use span_context::SpanContext;

#[cfg(feature = "telemetry-autoinit")]
use alloc::{boxed::Box, vec::Vec};
#[cfg(feature = "telemetry-autoinit")]
use greentic_telemetry::set_current_telemetry_ctx;
#[cfg(feature = "telemetry-autoinit")]
use tracing_subscriber::{Registry, layer::Layer};

#[cfg(feature = "telemetry-autoinit")]
pub use greentic_telemetry::init::TelemetryError;
#[cfg(feature = "telemetry-autoinit")]
pub use greentic_telemetry::{OtlpConfig, TelemetryCtx, init_otlp, layer_from_task_local};
#[cfg(feature = "telemetry-autoinit")]
pub use greentic_types_macros::main;
#[cfg(feature = "telemetry-autoinit")]
#[doc(hidden)]
pub use tokio::main as __tokio_main;

#[cfg(feature = "telemetry-autoinit")]
/// Installs the default Greentic telemetry stack using OTLP + task-local context injection.
pub fn install_telemetry(service_name: &str) -> Result<(), TelemetryError> {
    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".into());

    let layers: Vec<Box<dyn Layer<Registry> + Send + Sync + 'static>> =
        vec![Box::new(layer_from_task_local())];

    init_otlp(
        OtlpConfig {
            service_name: service_name.to_string(),
            endpoint: Some(endpoint),
            sampling_rate: None,
        },
        layers,
    )
}

#[cfg(feature = "telemetry-autoinit")]
/// Stores the tenant context into the task-local telemetry slot.
pub fn set_current_tenant_ctx(ctx: &crate::TenantCtx) {
    let mut telemetry = TelemetryCtx::new(ctx.tenant_id.as_ref());
    if let Some(session) = ctx.session_id() {
        telemetry = telemetry.with_session(session);
    }
    if let Some(flow) = ctx.flow_id() {
        telemetry = telemetry.with_flow(flow);
    }
    if let Some(node) = ctx.node_id() {
        telemetry = telemetry.with_node(node);
    }
    if let Some(provider) = ctx.provider_id() {
        telemetry = telemetry.with_provider(provider);
    }
    set_current_telemetry_ctx(telemetry);
}
