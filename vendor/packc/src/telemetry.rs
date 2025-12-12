pub use greentic_telemetry::TelemetryError;
pub use greentic_telemetry::with_task_local;
use greentic_telemetry::{
    OtlpConfig, TelemetryCtx, init_otlp, layer_from_task_local, set_current_telemetry_ctx,
};
use greentic_types::TenantCtx;
use tracing_subscriber::{Registry, layer::Layer};

/// Install the default Greentic telemetry stack for the given service.
pub fn install(service_name: &str) -> Result<(), TelemetryError> {
    let layers: Vec<Box<dyn Layer<Registry> + Send + Sync + 'static>> =
        vec![Box::new(layer_from_task_local())];

    init_otlp(
        OtlpConfig {
            service_name: service_name.to_string(),
            endpoint: None,
            sampling_rate: None,
        },
        layers,
    )
}

/// Map the provided tenant context into the task-local telemetry slot.
pub fn set_current_tenant_ctx(ctx: &TenantCtx) {
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
