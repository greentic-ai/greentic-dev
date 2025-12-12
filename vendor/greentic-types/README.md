# greentic-types

Shared primitives for Greentic’s next-generation runner, deployer, connectors, packs, and state/session backends.  
Every repository in the `greentic-ng` stack depends on these types to exchange tenant identity, session cursors, state pointers, policy decisions, pack references, and canonical error/outcome envelopes.

## Feature flags & MSRV

- **Default (`std`, `serde`, `time`, `otel-keys`)** – the recommended configuration for runners, CLIs, and tooling.
- **`schema`** – pulls in `schemars`, `anyhow`, and `serde_json` so you can call `write_all_schemas` or the `export-schemas` binary. (Derives continue to sit behind the lighter `schemars` feature for backwards compatibility.)
- **`otel-keys`** *(default)* – exposes `telemetry::OtlpKeys` and the schema for the OTLP attribute constants without requiring `telemetry-autoinit`.
- **`telemetry-autoinit`** – bundles the OTLP stack and task-local span helpers.
- **`uuid`** – adds UUID-based constructors for `SessionKey`.

MSRV: **Rust 1.85** (required by the 2024 edition). The MSRV is enforced in CI; when bumping it, update both `Cargo.toml` and the workflow matrix.

Disable defaults for fully `no_std` builds:
```toml
greentic-types = { version = "0.3.0", default-features = false, features = ["serde"] }
```

## Quickstart
```rust
use greentic_types::{
    AllowList, EnvId, ErrorCode, Outcome, SessionCursor, SessionKey, TenantCtx, TenantId,
};

let ctx = TenantCtx::new("prod".parse().unwrap(), "tenant-42".parse().unwrap())
    .with_team(Some("team-ops".parse().unwrap()))
    .with_user(Some("agent-7".parse().unwrap()));

let cursor = SessionCursor::new("node.entry")
    .with_wait_reason("awaiting human input")
    .with_outbox_marker("handoff");

let require_human: Outcome<()> = Outcome::Pending {
    reason: "Need operator approval".into(),
    expected_input: Some(vec!["approval".into()]),
};

let allow_policy = AllowList {
    domains: vec!["api.greentic.ai".into()],
    ports: vec![443],
    protocols: vec![greentic_types::Protocol::Https],
};

assert!(require_human.is_pending());
assert_eq!(cursor.node_pointer, "node.entry");
assert!(!allow_policy.is_empty());
```

### Tenant UI/discovery documents
`tenant_config` holds serde-ready shapes for the public tenant documents: `RepoSkin` (`skin.json`), `RepoAuth` (`auth.json`), `RepoTenantConfig` (`config.json` with free-form tab/handler keys), and `TenantDidDocument` (`did.json` with flexible `@context`). These stay UI-neutral (strings for tabs/slots) so UI/server crates can evolve without bumping greentic-types.

### Sessions & telemetry
```rust
use greentic_types::{SessionKey, SpanContext};

let session = SessionKey::generate();
let span = SpanContext::new("tenant-42".into(), "flow-welcome", "runner-core")
    .with_session(session.clone())
    .with_node("node.entry");
```

### Outcomes & errors
```rust
use greentic_types::{ErrorCode, GreenticError, Outcome};

fn validate(input: &str) -> greentic_types::GResult<()> {
    if input.is_empty() {
        return Err(GreenticError::new(ErrorCode::InvalidInput, "missing payload"));
    }
    Ok(())
}

let outcome = match validate("payload") {
    Ok(_) => Outcome::Done("payload accepted"),
    Err(err) => Outcome::Error {
        code: err.code,
        message: err.message.clone(),
    },
};
```

### Schema generation & publishing
```rust
#[cfg(feature = "schemars")]
{
    let schema = schemars::schema_for!(greentic_types::TenantCtx);
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}

#[cfg(feature = "schema")]
{
    use std::path::Path;
    greentic_types::write_all_schemas(Path::new("dist/schemas/v1"))?;
}
```
- `cargo run --bin export-schemas --all-features` runs the helper binary and writes JSON Schemas into `dist/schemas/v1/`.
- Published schemas (and canonical URLs) live in [SCHEMAS.md](SCHEMAS.md); CI pushes them to GitHub Pages automatically.

### Worker envelope
- `WorkerRequest`, `WorkerResponse`, and `WorkerMessage` are the shared worker envelope models, mirroring the `greentic:worker@1.0.0` WIT package.
- They are domain-agnostic (no repo/store/channel concepts) and are shared between runner and messaging. See [docs/worker.md](docs/worker.md) for field breakdowns and usage notes.

## Telemetry (auto-init)
- Enable with `features = ["telemetry-autoinit"]` to bundle the OTLP stack and entry-point macro.
- `#[greentic_types::telemetry::main(...)]` wraps `tokio::main`, installs OTLP once, and forwards to your async main.
- `install_telemetry("name")` is available if you need to wire custom runtimes or tests manually.
- Uses `OTEL_EXPORTER_OTLP_ENDPOINT` when set (defaults to `http://localhost:4317`).
- `set_current_tenant_ctx(&TenantCtx)` maps the current tenant into the task-local telemetry slot.

```rust
use greentic_types::{EnvId, TenantCtx, TenantId};
use greentic_types::telemetry::set_current_tenant_ctx;

#[greentic_types::telemetry::main(service_name = "greentic-runner")]
async fn main() -> anyhow::Result<()> {
let ctx = TenantCtx::new("prod".parse().unwrap(), "acme".parse().unwrap())
    .with_session("s1")
    .with_flow("hello")
    .with_node("qa-1")
    .with_provider("runner");
    set_current_tenant_ctx(&ctx);
    tracing::info!("tenant-aware spans now have gt.* attrs");
    Ok(())
}
```

## Flow, component, and pack models
- [`Flow`, `Node`, `Routing`, `FlowKind`](src/flow.rs) form the unified flow model (Greentic Flow v1) with structured routing, telemetry hints, and component refs using ID newtypes; nodes remain insertion-ordered so the first entry is the ingress.
- [`ComponentManifest`](src/component.rs) captures capabilities, configurators, operations, optional config schema, resource hints, and profile helpers like `select_profile`.
- [`PackManifest`](src/pack_manifest.rs) embeds flows directly via `PackFlowEntry`, includes components, dependencies, capabilities, signatures, and the new `PackKind` (`application`, `provider`, `infrastructure`, `library`).
- `HostCapabilities` expose generic IaC toggles via the `iac` block so deployment components can request template write/execute access without encoding provider details.
- [`DeploymentPlan`](src/deployment.rs) stays the shared, provider-neutral description of what needs to run for a tenant/environment.

Read [MODELS.md](MODELS.md) for the guiding principles: IDs are opaque strings, capabilities describe host/WASI interaction, and bindings stay outside these documents.

## Harmonised model
- **TenantCtx & TenantIdentity** – shared across runner, connectors, and state/session stores; keeps legacy (`tenant`, `team`, `user`) and next-gen (`tenant_id`, `team_id`, `user_id`, `impersonation`) fields aligned.
- **SessionKey/SessionCursor** – referenced by session routers and state stores.
- **StateKey/StatePath** – JSON pointer compatible navigation for persisted state.
- **Outcome<T> & GreenticError** – canonical execution envelope for nodes, adapters, and tools.
- **AllowList/NetworkPolicy & PolicyDecision** – security model used by deployer and connector sandboxes.
- **PackRef/Signature** – pack registry references with deterministic semver + base64 signatures.
- **SpanContext** – OTLP-aligned telemetry context (tenant, flow, node, provider, start/end).

## Working with other crates
- `greentic-runner`, `greentic-session-store`, `greentic-state-store`, `greentic-deployer`, `greentic-connectors`, and `greentic-packs` depend on this crate. Always add new shared types here first to avoid duplication.
- ABI/WIT contracts live in **greentic-interfaces**; never re-define those types here.

### Adopt in other repos
- Replace bespoke definitions of `RunResult`, `RunStatus`, `NodeStatus`, `NodeSummary`, `NodeFailure`, `Capabilities`, `Limits`, `TelemetrySpec`, the ID newtypes (`PackId`, `FlowId`, etc.), and now the pack/flow/component schemas with the versions exported by `greentic-types`.
- Hook your manifests, CLIs, and IDE tooling to the canonical schema URLs from [SCHEMAS.md](SCHEMAS.md) for validation.
- Add a dependency on this crate (with the appropriate features) before introducing new shared structs so CI can enforce the no-duplicate rule.

## Development workflow
```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test --all-features
```

### Local CI mirror
Before pushing, run the bundled CI mirror which replicates the GitHub Actions steps:

```bash
ci/local_check.sh
```

Toggles:

- `LOCAL_CHECK_ONLINE=1` – enable networked checks (schema URL curl).
- `LOCAL_CHECK_STRICT=1` – fail when required tools (rustup, curl, etc.) are missing.
- `LOCAL_CHECK_VERBOSE=1` – print every command before it runs.

Example:

```bash
LOCAL_CHECK_ONLINE=1 LOCAL_CHECK_STRICT=1 ci/local_check.sh
```

Tip: add a Git hook to run it automatically:

```bash
ln -s ../../ci/local_check.sh .git/hooks/pre-push
```

CI (see `.github/workflows/publish.yml`) enforces the same gates on push/PR. Legacy `v*` tags still trigger the workflow alongside per-crate tags.

## Maintenance notes
- All public structs derive `schemars::JsonSchema` when the feature is enabled; integration tests assert schema registration and serde round-trips.
- `GResult<T>` aliases `Result<T, GreenticError>` for consistent error propagation.
- Prefer zero-copy APIs; the crate is `#![forbid(unsafe_code)]`.
- Use feature flags to keep downstream binaries lightweight (e.g., disable `uuid` in constrained connectors).
- The crate version is exposed at runtime via `greentic_types::VERSION` for telemetry banners or capability negotiation.

## Releases & Publishing
- Versions come directly from each crate’s `Cargo.toml`.
- Pushing to `master` tags any crate whose version changed in that commit using `<crate-name>-v<semver>`.
- After tagging (or even when no tags are created), the publish workflow attempts to publish all changed crates via `katyo/publish-crates@v2`.
- Publishing is idempotent; attempting to release the same version again succeeds without errors.


## License
MIT License. See [LICENSE](LICENSE).
