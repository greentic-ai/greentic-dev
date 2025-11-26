use greentic_interfaces_guest::{
    component,
    component::exports::greentic::component::node,
    http_client,
    secrets_store,
    state_store,
    telemetry_logger,
    types,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

struct Component;

component::export!(Component);

#[derive(Debug, Deserialize)]
struct InvokeInput {
    #[serde(default)]
    message: String,
    #[serde(default = "default_secret_name")]
    secret_name: String,
}

#[derive(Debug, Serialize)]
struct InvokeOutput {
    echoed: String,
    operation: String,
    secret_sample: Option<String>,
    state_value: Option<String>,
    http_status: Option<u16>,
}

impl node::Guest for Component {
    fn get_manifest() -> String {
        let schema: Value = serde_json::from_str(include_str!("../schemas/v1/config.schema.json"))
            .expect("config schema json to be valid");
        json!({
            "name": "{{component_name}}",
            "description": "Generated Greentic component scaffold",
            "capabilities": ["telemetry", "secrets", "state", "http"],
            "exports": [
                { "operation": "invoke" }
            ],
            "config_schema": schema,
            "secrets": ["{{component_name}}/example"],
            "wit_compat": {
                "package": "greentic:component",
                "min": "{{component_world_version}}",
                "max": "{{component_world_version}}"
            }
        })
        .to_string()
    }

    fn on_start(ctx: node::ExecCtx) -> Result<node::LifecycleStatus, String> {
        let tenant_ctx = tenant_ctx_from_exec(&ctx);
        let span = span_from_exec(&ctx);
        let _ = telemetry_logger::log(
            span,
            vec![("event".to_string(), "component-start".to_string())],
            Some(tenant_ctx),
        )
        .map_err(format_host_error);
        Ok(node::LifecycleStatus::Ok)
    }

    fn on_stop(
        ctx: node::ExecCtx,
        reason: String,
    ) -> Result<node::LifecycleStatus, String> {
        let tenant_ctx = tenant_ctx_from_exec(&ctx);
        let span = span_from_exec(&ctx);
        let _ = telemetry_logger::log(
            span,
            vec![
                ("event".to_string(), "component-stop".to_string()),
                ("reason".to_string(), reason),
            ],
            Some(tenant_ctx),
        )
        .map_err(format_host_error);
        Ok(node::LifecycleStatus::Ok)
    }

    fn invoke(
        ctx: node::ExecCtx,
        operation: String,
        input: String,
    ) -> node::InvokeResult {
        let payload: InvokeInput =
            serde_json::from_str(&input).unwrap_or(InvokeInput { message: input, secret_name: default_secret_name() });
        let tenant_ctx = tenant_ctx_from_exec(&ctx);

        let secret_sample = secrets_store::read(&payload.secret_name)
            .ok()
            .and_then(Result::ok)
            .and_then(|bytes| String::from_utf8(bytes).ok());

        let state_key: types::StateKey = format!("{{component_name}}:echo").into();
        let state_value = state_store::read(state_key.clone(), Some(tenant_ctx.clone()))
            .ok()
            .and_then(Result::ok)
            .and_then(|bytes| String::from_utf8(bytes).ok());

        let http_status = http_client::send(
            http_client::Request {
                method: "GET".to_string(),
                url: "https://example.com/health".to_string(),
                headers: Vec::new(),
                body: None,
            },
            Some(tenant_ctx.clone()),
        )
        .ok()
        .and_then(Result::ok)
        .map(|resp| resp.status);

        let response = InvokeOutput {
            echoed: payload.message,
            operation,
            secret_sample,
            state_value,
            http_status,
        };
        node::InvokeResult::Ok(
            serde_json::to_string(&response).expect("serialize response"),
        )
    }

    fn invoke_stream(
        _ctx: node::ExecCtx,
        _operation: String,
        _input: String,
    ) -> Vec<node::StreamEvent> {
        Vec::new()
    }
}

fn tenant_ctx_from_exec(ctx: &node::ExecCtx) -> types::TenantCtx {
    let tenant = ctx.tenant.tenant.clone();
    types::TenantCtx {
        env: tenant.clone(),
        tenant: tenant.clone(),
        tenant_id: tenant.clone(),
        team: ctx.tenant.team.clone(),
        team_id: ctx.tenant.team.clone(),
        user: ctx.tenant.user.clone(),
        user_id: ctx.tenant.user.clone(),
        session_id: None,
        flow_id: Some(ctx.flow_id.clone()),
        node_id: ctx.node_id.clone(),
        provider_id: None,
        trace_id: ctx.tenant.trace_id.clone(),
        correlation_id: ctx.tenant.correlation_id.clone(),
        attributes: Vec::new(),
        deadline_ms: ctx
            .tenant
            .deadline_unix_ms
            .and_then(|d| i64::try_from(d).ok()),
        attempt: ctx.tenant.attempt,
        idempotency_key: ctx.tenant.idempotency_key.clone(),
        impersonation: None,
    }
}

fn span_from_exec(ctx: &node::ExecCtx) -> types::SpanContext {
    types::SpanContext {
        tenant: ctx.tenant.tenant.clone(),
        session_id: None,
        flow_id: ctx.flow_id.clone(),
        node_id: ctx.node_id.clone(),
        provider: "{{component_name}}".to_string(),
        start_ms: None,
        end_ms: None,
    }
}

fn format_host_error<E: std::fmt::Display>(err: E) -> String {
    err.to_string()
}

fn default_secret_name() -> String {
    "{{component_name}}/example".to_string()
}
