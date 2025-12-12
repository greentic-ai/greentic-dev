# 🧩 Greentic Core Design Manifesto
*Shared Principles for All Greentic Repositories and Future Components*

---

## 1. Core Philosophy
Greentic is a **modular, multi-tenant, multi-platform agentic runtime**.

Each domain (e.g., `secrets`, `messaging`, `oauth`, `telemetry`, `config`, `timer`, `webhook`, etc.) has:

- A **core crate** defining its WIT interfaces and Rust traits.  
- **Independent provider crates** (one per platform or cloud).  
- A **shared type system** (`greentic-types`) for tenancy, deployment, and tracing.  
- A **shared interface package** (`greentic-interfaces`) for WIT + Rust traits.

> **Golden Rule:**  
> All components must speak the same *contract* (WIT / trait).  
> Implementation form (WASM, native, sidecar, or remote) may vary,  
> but the behaviour, context, and telemetry must remain identical.

---

## 2. Repository Structure

| Layer | Responsibility | Example Repositories |
|-------|----------------|----------------------|
| **Types** | Shared structs, IDs, tenancy, error types | `greentic-types` |
| **Interfaces** | WIT + Rust traits for all domains | `greentic-interfaces` |
| **Runtime** | Component host, provider registry, routing, policies | `greentic-runtime` |
| **Domain crates** | Domain logic + provider registry | `greentic-secrets`, `greentic-messaging`, `greentic-oauth`, `greentic-telemetry`, `greentic-config`, `greentic-timer`, `greentic-webhook` |
| **Provider crates** | Cloud / platform implementations | `greentic-secrets-aws`, `greentic-messaging-teams`, `greentic-oauth-google` |
| **Conformance tests** | Shared validation for all providers | `greentic-conformance` |

---

## 3. Multi-Tenancy Model

All interfaces accept a **`TenantCtx`**:

```rust
pub struct TenantCtx {
  pub tenant: TenantId,
  pub team: Option<TeamId>,
  pub user: Option<UserId>,
  pub deployment: DeploymentCtx,
  pub trace_id: Option<String>,
}
```

### Deployment Context

```rust
pub struct DeploymentCtx {
  pub cloud: Cloud,            // Aws | Gcp | Azure | Hetzner | Local | Other(String)
  pub region: Option<String>,
  pub platform: Platform,      // K8s | Nomad | Systemd | CFWorkers | Lambda | Baremetal
  pub runtime: Option<String>, // wasmtime, firecracker, etc.
}
```

**Every call must be scoped and traceable** by tenant, team, user, and platform.

---

## 4. Provider Design

Each provider implements a domain trait, e.g.:

```rust
#[async_trait::async_trait]
pub trait MessagingProvider: Send + Sync {
  fn name(&self) -> &'static str;
  async fn send(&self, ctx: &TenantCtx, msg: SendMessage)
      -> Result<SendResult, MessagingError>;
}
```

### Provider Implementation Forms

| Form | Description | When to Use |
|------|--------------|-------------|
| **WASM Component** | Runs as Wasmtime component implementing WIT | Async-only, pure Rust |
| **Native In-Proc** | Linked crate registered in runtime | Heavy Tokio/Tower deps |
| **Sidecar Service** | Separate process or Deployment | Polyglot / large SDKs |
| **Remote Gateway** | Cloud-hosted connector | SaaS or shared connectors |

All share the same contract and self-description manifest.

---

## 5. Provider Manifest (Self-Describing)

Every provider publishes a `provider.json` or `.toml`:

```json
{
  "name": "greentic-messaging-teams",
  "domain": "messaging",
  "capabilities": ["send", "reply", "adaptive-cards"],
  "transport": {"kind": "grpc", "path": "/Provider/Invoke"},
  "config_schema": {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "properties": {"tenant_app_id": {"type": "string"}},
    "required": ["tenant_app_id"]
  },
  "secrets": ["MSGRAPH_CLIENT_ID", "MSGRAPH_CLIENT_SECRET", "MSGRAPH_TENANT_ID"],
  "compat": {"wit": "greentic:messaging/provider@1.2.x"}
}
```

Runtimes and Operators use this to:
- Validate configs against schema  
- Discover capabilities  
- Resolve secrets  
- Enforce WIT compatibility  

---

## 6. Platform & Deployment Principles

### Kubernetes (Reference Deployment)
- **Namespace = Tenant**, **ServiceAccount = Team**
- **CRDs:** `ProviderClass`, `ProviderBinding`, `RoutingPolicy`
- **Operator:** validates and reconciles manifests
- **Secrets:** via External Secrets Operator (Vault/KMS)
- **mTLS:** cert-manager or mesh (Linkerd/Istio)
- **NetworkPolicy:** runtime ↔ provider only
- **Autoscale:** KEDA based on NATS/OTLP metrics
- **Telemetry:** OpenTelemetry Collector → Elastic, CloudWatch, or other

### Other Platforms

| Platform | Secret / Config | Auth / Mesh | Deployment |
|-----------|-----------------|--------------|-------------|
| **Nomad** | Vault | Consul Connect | Job specs |
| **Systemd** | .env + Vault agent | Local mTLS | Services |
| **CF Workers** | CF KV / R2 | JWT | Workers |
| **Bare Metal** | .env + Vault | Local PKI | Daemons |

All must use the same `DeploymentCtx` schema.

---

## 7. Runtime Behaviour

- **Registry:** `DashMap<String, Arc<dyn Provider>>`
- **RoutingPolicy:** chooses provider per `{tenant, team, user, cloud, capability}`
- **Invocation Flow:**  
  `FlowNode → ProviderBinding → ProviderAdapter (InProc/Wasm/RPC) → Provider`
- **Resilience:** retries, exponential backoff, circuit breakers
- **Telemetry:** OTLP spans/logs with `{tenant, team, user, provider, capability}`
- **Security:** JWT + mTLS for all inter-component calls
- **Secrets:** resolved at call-time via `greentic-secrets`

---

## 8. Conformance & Testing

Each domain has a shared conformance suite verifying:

- Error mapping consistency  
- Tenancy scoping (no leakage)  
- Idempotency  
- Retry & backoff behaviour  
- Telemetry emission  
- Secret resolution  

### Example CI usage
```yaml
if: env.PROVIDER_CREDENTIALS
run: cargo test --features conformance
```

---

## 9. Extensibility Guidelines

When adding a new domain (e.g., timer, webhook, oauth, storage, ai):

1. Define WIT + Rust trait in `greentic-interfaces`.  
2. Add a `greentic-{domain}` crate for the registry and adapters.  
3. Create providers as separate repos: `greentic-{domain}-{provider}`.  
4. Publish manifest and (optional) Helm chart.  
5. Add conformance tests using `greentic-conformance`.  
6. Pass `TenantCtx` + `DeploymentCtx` through all APIs.  
7. Emit metrics and tracing via `greentic-telemetry`.  
8. Follow naming:  
   - `greentic-{domain}` → domain crate  
   - `greentic-{domain}-{provider}` → provider crate  
   - `greentic-{domain}-spec` → optional WIT/spec crate  
9. Register via `register_provider()` at runtime or through Operator CRD.

---

## 10. Security & Compliance

- **Zero-trust default:** explicit capability grant per provider.  
- **mTLS everywhere** (mesh or direct).  
- **No secrets persisted**; ephemeral injection only.  
- **Audit logs signed + streamed** via `greentic-telemetry`.  
- **SBOM + cosign** signing for all artifacts (WASM, container, binary).  
- **Multi-tenant isolation** via namespaces, quotas, and network policies.

---

## 11. Guiding Principles Summary

| Principle | Description |
|------------|-------------|
| **Contract before code** | Define WIT + types first |
| **Provider isolation** | One repo per provider |
| **Multi-tenant first** | `TenantCtx` always required |
| **Multi-platform** | Works on K8s, Nomad, Systemd, CF Workers, Baremetal |
| **Manifest = truth** | Providers are self-describing |
| **Runtime agnostic** | InProc, WASM, Sidecar, Remote |
| **Test once, certify everywhere** | Shared conformance suite |
| **Security & telemetry built-in** | mTLS, OTLP, audit required |
| **Composable flows** | Nodes reuse provider interface patterns |

---

## 12. Example: Adding `greentic-timer`

✅ Add WIT + trait → `greentic-interfaces`  
✅ Create `greentic-timer` crate → registry + adapters  
✅ Provider repos → `greentic-timer-k8s`, `greentic-timer-nomad`, `greentic-timer-local`  
✅ Include manifest + Helm chart  
✅ Add conformance tests  
✅ Add `TimerNode` in `greentic-runtime`  
✅ Ensure `TenantCtx` and `DeploymentCtx` are passed through  

---

### 💡 TL;DR for Future Contributors / AI Sessions
> All new Greentic repos follow the Manifesto:
> - Traits + WIT → `greentic-interfaces`; shared structs → `greentic-types`.
> - Providers live in separate repos; describe themselves via manifest + schema.
> - All APIs are multi-tenant (`TenantCtx`) and platform-aware (`DeploymentCtx`).
> - Implementation may be WASM, native, sidecar, or remote—same contract.
> - Kubernetes is the reference platform (Operator, CRDs, ESO, OTEL).
> - Security, telemetry, and conformance testing are mandatory.

---

**Version:** 1.0  
**Maintainer:** Greentic-AI Architecture Group  
**Last Updated:** 2025-10-29
