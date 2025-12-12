# 🧩 Greentic Pack, Flow & Component Standard (2025-10 Draft)

This document summarises the generic data model implemented in `greentic-types` and referenced by every other Greentic repository. It mirrors the “Greentic Pack, Flow & Component Standards (Draft v1)” spec and the new Rust types:

- `FlowKind`, `Flow`, `Node` (flows/.ygtc)
- `ComponentManifest` + capability structs
- `PackManifest`, `PackFlowRef`, `PackComponentRef`

Use this as the authoritative overview when writing docs, CLIs, or tooling.

---

## 1. Flows (`.ygtc`)

Flows are tiny YAML DAGs. The schema is intentionally minimal so humans and small LLMs can author them safely.

```yaml
kind: messaging          # or: events
id: demo.messaging.flow
description: Optional summary
nodes:
  router:
    kind: process/router          # opaque string interpreted by the component/runtime
    component: vendor.router      # optional; ties to ComponentManifest::id
    profile: decision-profile     # optional override
    config: { mode: "intent" }    # free-form JSON/YAML blob
    routing:                      # arbitrary JSON/YAML; component decides semantics
      default: handler

  handler:
    kind: component-kind-1
    component: vendor.component.one
    config:
      reply_text: "ack"
    routing:
      default: finish

  finish:
    kind: messaging/reply         # still opaque; just a string
    config:
      text: "Done"
```

Key rules:

1. **Flow kinds** — Only `messaging` and `events`. Future ingress modes are implemented via components + connectors, not additional flow types.
2. **Opaque identifiers** — `NodeId`, `component`, `kind`, and `profile` are all strings. There are no enums like “qa” or “tool” baked into the schema.
3. **Insertion order matters** — `Flow.nodes` is an ordered map. The first node is the implicit ingress (no `webhook.in` or `teams.in` node required).
4. **Routing is component-defined** — `routing` is `serde_json::Value` in the Rust model. Components decide whether it’s `{default: "next"}` or a richer object.

`Flow::validate_components` in `greentic-types` ensures:
- At least one node exists.
- Referenced component manifests exist and list the same `FlowKind`.

---

## 2. Component Manifests

`ComponentManifest` describes a reusable WASM module and how it interacts with the host. All identifiers remain opaque.

```yaml
id: vendor.component.qa
version: 1.2.3
supports: [messaging]
world: "vendor:component@1.0.0"
profiles:
  default: stateless
  supported: [stateless, cached]
capabilities:
  wasi:
    random: true
    clocks: true
    filesystem:
      mode: sandbox
      mounts:
        - name: scratch
          host_class: scratch
          guest_path: /tmp
    env:
      allow: [RUST_LOG]
  host:
    secrets:
      required: [API_TOKEN]
    messaging:
      inbound: true
      outbound: true
    telemetry:
      scope: tenant
configurators:
  basic: configure_component_basic
  full: configure_component_full
```

Important fields:

- `supports` — `FlowKind`s where the component can run.
- `profiles` — Named capability bundles. `ComponentManifest::select_profile` handles validation/defaults.
- `capabilities.wasi` — Filesystem/env/random/clock toggles. Filesystem mode is an enum (`None`, `ReadOnly`, `Sandbox`).
- `capabilities.host` — `secrets`, `state`, `messaging`, `events`, `http`, `telemetry`. Each struct only expresses interaction patterns (e.g., `messaging.inbound = true`), never business semantics.
- `configurators` — Optional flows (identified by `FlowId`) that let DX tools run “basic” or “full” configuration sessions. These are regular messaging flows.

---

## 3. Pack Manifests (`PackManifest`)

Thin packs reference flows and components, plus optional profile defaults or connector metadata. They no longer store bindings or component semantics.

```yaml
id: vendor.demo.pack
version: 0.1.0
name: "Demo Pack"

flows:
  - id: demo.messaging.flow
    file: flows/messaging.ygtc

components:
  - id: vendor.component.qa
    version_req: "^1.2"
    source: "oci://registry/components"

profiles:
  messaging:
    defaults:
      handler: stateless

component_sources:
  registry: "greentic-store"

connectors:
  messaging:
    teams:
      flow: demo.messaging.flow
      channel: "support"
```

Notes:

- `flows` only lists `id` + relative file path; the flow itself carries its kind/nodes.
- `components` reference `ComponentManifest`s indirectly via `id` + `version_req` (which uses the `SemverReq` helper). `source` is optional (OCI, registry alias, etc.).
- `profiles`, `component_sources`, `connectors` are intentionally `serde_json::Value` blobs so tenants can decide their own shapes. Tooling should pass them through untouched.

`PackManifest` exposes no binding fields. Hosts generate bindings at runtime using capabilities, profile selection, tenant policy, and environment-specific defaults.

---

## 4. Runtime Responsibilities (Recap)

1. **Load pack** — Parse `PackManifest`, load `.ygtc` flows as `Flow`.
2. **Resolve components** — Fetch `ComponentManifest`s (local `components/` dir, registry resolver trait, etc.).
3. **Validate flows** — Ensure nodes exist, components support the flow kind, routing references valid nodes (when the component uses simple maps).
4. **Aggregate capabilities** — Combine component manifest + selected profile (node override → component default) to produce `ComponentCapabilities`.
5. **Generate bindings** — Hosts such as `greentic-pack` or `greentic-runner` convert capabilities into WASI/host bindings (filesystem mounts, env allow-lists, secrets, etc.). Strict vs. complete binding modes determine whether defaults (scratch dirs, `RUST_LOG`) are injected.
6. **Execute** — The runner instantiates each node’s WASM component with the generated bindings, feeds it the current payload, and uses the returned route label (plus `node.routing`) to choose the next `NodeId`.

None of these steps rely on domain-specific node kinds; everything hinges on the opaque strings declared in the flow.

---

## 5. Developer Workflow (recap)

1. Create a pack folder (`manifest.yaml`, `flows/`, optional `components/`).
2. Author `.ygtc` flows using the schema above.
3. Reference component manifests via `components` entries (bundled or external).
4. Run CLI helpers (`greentic-pack inspect`, `greentic-pack validate`, `greentic-pack capabilities`, `greentic-pack bindings`).
5. Distribute packs via OCI or git; hosts resolve components and execute flows using the shared types.

---

## 6. Alignment Checklist

- [ ] Flows only declare `kind`, `id`, `description`, ordered `nodes` (with `kind/profile/component/config/routing`).
- [ ] Components only express host/WASI capabilities; no semantics like “qa” or “rag”.
- [ ] Packs only reference flows/components and carry optional opaque metadata.
- [ ] Bindings and connector wiring stay outside of packs/flows/components.
- [ ] Docs/tools reference the canonical schemas from `SCHEMAS.md`.

When in doubt, read `MODELS.md` and the source definitions in `src/flow.rs`, `src/component.rs`, and `src/pack_manifest.rs`.
