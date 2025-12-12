# Greentic Pack / Flow / Component Specification

**Version:** 1.0 (Draft – Deployment & OAuth integrated)  
**Audience:** Engineers, tool authors, Codex prompts  
**Status:** Canonical working draft for Greentic-NG repos

---

## Table of Contents

1. [Scope & Goals](#1-scope--goals)  
2. [Core Design Principles & Invariants](#2-core-design-principles--invariants)  
3. [High-Level Data Model](#3-high-level-data-model)  
4. [Flow Specification (`.ygtc`)](#4-flow-specification-ygtc)  
   - 4.1 [Flow Types](#41-flow-types)  
   - 4.2 [Structure & Schema](#42-structure--schema)  
   - 4.3 [Routing Semantics](#43-routing-semantics)  
   - 4.4 [Validation Rules](#44-validation-rules)  
5. [Component Specification](#5-component-specification)  
   - 5.1 [Component Manifest](#51-component-manifest)  
   - 5.2 [Capabilities](#52-capabilities)  
   - 5.3 [Profiles](#53-profiles)  
   - 5.4 [Configurator Flows](#54-configurator-flows)  
   - 5.5 [WIT Worlds](#55-wit-worlds)  
6. [Pack Specification (`.gtpack`)](#6-pack-specification-gtpack)  
   - 6.1 [Directory Layout](#61-directory-layout)  
   - 6.2 [Pack Manifest & `PackKind`](#62-pack-manifest--packkind)  
   - 6.3 [Connectors](#63-connectors)  
7. [Deployment Extension](#7-deployment-extension)  
   - 7.1 [DeploymentPack vs ApplicationPack](#71-deploymentpack-vs-applicationpack)  
   - 7.2 [DeploymentPlan](#72-deploymentplan)  
   - 7.3 [Deployment Flows (Events Flows)](#73-deployment-flows-events-flows)  
   - 7.4 [WIT `greentic:deploy-plan@1.0.0`](#74-wit-greenticdeploy-plan100)  
   - 7.5 [IaC Capabilities](#75-iac-capabilities)  
8. [OAuth Extension](#8-oauth-extension)  
   - 8.1 [Host Capability: OAuth](#81-host-capability-oauth)  
   - 8.2 [WIT `greentic:oauth-broker@1.0.0`](#82-wit-greenticoauth-broker100)  
   - 8.3 [Token Semantics (Opaque)](#83-token-semantics-opaque)  
9. [Bindings Generation](#9-bindings-generation)  
   - 9.1 [Inputs](#91-inputs)  
   - 9.2 [Pipeline Steps](#92-pipeline-steps)  
   - 9.3 [Debug/Inspection](#93-debuginspection)  
10. [Execution Model](#10-execution-model)  
    - 10.1 [Messaging Flows](#101-messaging-flows)  
    - 10.2 [Events Flows](#102-events-flows)  
    - 10.3 [Deployment Flows](#103-deployment-flows)  
11. [Tool Responsibilities](#11-tool-responsibilities)  
12. [Conformance Expectations](#12-conformance-expectations)  
13. [Glossary](#13-glossary)  
14. [Worked Examples](#14-worked-examples)  

---

## 1. Scope & Goals

Greentic-NG aims to support:

- **Billions** of autonomous flows  
- **Tens of thousands** of reusable WASM components  
- Multi-tenant, multi-team, multi-user execution  
- Cloud, edge, serverless, and on-prem runners  
- Tiny, LLM-friendly flow definitions  

This specification describes the **shared model**:

- **Flows** (`.ygtc`) — logical DAGs only  
- **Components** — WASM + capability manifests + configurators  
- **Packs** (`.gtpack`) — bundles of flows + component references  
- **DeploymentPlan** — generic description of “what needs to exist”  
- **Host capabilities** — secrets, state, messaging, events, http, telemetry, oauth, iac  

Everything environment-specific (bindings, paths, credentials, providers, clusters, etc.) lives **outside** these artifacts and is derived dynamically at runtime.

---

## 2. Core Design Principles & Invariants

These invariants must **never** be violated by shared types or core tools.

### 2.1 Opaque Semantics

Identifiers are **opaque strings**:

- Component IDs (e.g. `greentic.agent.qa`, `greentic.deploy.generic.iac`)  
- Node kinds (e.g. `process/router`, `deploy.renderer`, `http.out`)  
- Flow IDs (e.g. `support_triage`, `deploy_generic_iac`)  
- Channel kinds (e.g. `teams`, `webchat`, `webhook`)

The shared model NEVER encodes “this is a QA agent” or “this is an AWS deployment”.  
Semantics belong to:

- Component implementations  
- Host-side integrations  
- Documentation and comments  

### 2.2 Flows Stay Tiny

Flows are **pure logical graphs**:

- `type` (messaging | events)  
- `id`, `description`  
- `nodes` (map of node-id → node definition)  
- `routing` (per node)

Flows **do not contain**:

- Bindings (WASI preopens, env vars, host imports)  
- Secrets or OAuth credentials  
- Provider-specific information  
- File system paths or URLs (beyond node-local config fields)

This keeps flows:

- Easy for humans to read  
- Easy for small LLMs to generate  
- Independent of deployment target, environment, or provider  

### 2.3 Components Describe Interaction, Not Purpose

Components declare:

- **Capabilities** — what host services they need (secrets, messaging, http, etc.)  
- **Profiles** — bundles of capabilities for common usage patterns  
- **WIT worlds** — interfaces imported from the host  
- **Configurator flows** — interactive configuration helpers  

Components **do not encode** in shared types:

- “QA agent”, “RAG engine”, “router”, “deploy-to-AWS”  
- Any behaviour beyond “I need these host services”

### 2.4 Packs Are Thin and Portable

Packs contain:

- Flow files  
- Component references (IDs + version constraints)  
- Optional embedded components  
- Optional assets (cards, prompts, icons)  
- Optional `kind` hint (`application`, `deployment`, `mixed`)  

Packs DO NOT encode:

- Bindings, secrets, providers, cluster IDs, regions, etc.

The same pack can run:

- On different clouds  
- In dev/staging/prod  
- With different binding policies  

### 2.5 Bindings Are Always Generated

Bindings:

- Define how a component sees WASI + host capabilities  
- Are generated per node at runtime  
- Are derived from capabilities, profiles, and policies  
- Are never authored by humans or stored in packs/flows  

---

## 3. High-Level Data Model

At a high level:

- **Flow (.ygtc → Flow)** — a small DAG of nodes, each node referencing a component (with optional operation), input/output mappings, structured routing (`Routing` enum), and optional telemetry hints.
- **ComponentManifest** — metadata and capability description for a WASM module.
- **Pack (.gtpack)** — a directory/archive containing a manifest with embedded flows and component manifests.
- **DeploymentPlan** — a provider-agnostic plan describing “desired runtime topology”.
- **Bindings** — derived, ephemeral host configuration for each node.
- **TenantCtx** — tenant/team/user identity and scoping context (defined in `greentic-types`).

---

## 4. Flow Specification (`.ygtc` → Flow)

Flows are YAML files describing logical graphs compiled into the unified Flow model.

### 4.1 Flow Kinds

Each flow has exactly one kind:

```yaml
kind: messaging | event | component_config | job | http
```

- **`messaging`** — session-based messaging (Teams, WebChat, Slack, etc.).  
- **`event`** — fire-and-forget events (webhook, timer, email-in, file-in, queue).  
- **`component_config`** — flows that configure components/providers/infra.  
- **`job`** — batch/background jobs.  
- **`http`** — request/response style flows.

No additional types are introduced for deployment or OAuth.

---

### 4.2 Structure & Schema

Minimal conceptual schema (YAML → Flow):

```yaml
kind: messaging | event | component_config | job | http
schema_version: flow-v1
id: <flow-id>
entrypoints:
  default: {}
  telegram: {}
nodes:
  <node-id>:
    id: <node-id>
    component:
      id: <component-id>
      pack_alias: <optional dependency alias>
      operation: <optional operation>
    input:
      mapping: <arbitrary mapping object>
    output:
      mapping: <arbitrary mapping object>
    routing:
      next:
        node_id: <node-id>
      # or branch/end/reply/custom
    telemetry:
      span_name: <optional string>
      attributes: { key: value, ... }
      sampling: <optional string>
metadata:
  title: <optional>
  description: <optional>
  tags: [foo, bar]
  extra: {}
```

#### 4.2.1 Node IDs

- `node-id` is a simple string: e.g., `router`, `qa`, `fallback`, `render`, `done`.
- Must be unique within a flow.

#### 4.2.2 Component Kind

- `component-kind` is an opaque string: e.g., `process/router`, `agent/qa`, `http.out`, `deploy.renderer`.
- Flows **do not** interpret this value; only components + tooling do.

#### 4.2.3 Component Reference

`component.id` references a component manifest entry; `pack_alias` points at dependency packs; `operation` names an operation within the component (optional).

#### 4.2.4 Mappings

`input.mapping` / `output.mapping` are opaque JSON objects used by tooling/runtimes to map payloads/context.

#### 4.2.5 Routing

`routing` is structured via the `Routing` enum: `next { node_id }`, `branch { on_status, default }`, `end`, `reply`, or `custom` (arbitrary JSON).

- Structure is **component-defined**.
- The spec does not constrain fields beyond “must be valid YAML/JSON”.

---

### 4.3 Routing Semantics

Routing is **local to each node** and uses the structured `Routing` enum:

- `next { node_id }`
- `branch { on_status: {status -> node}, default }`
- `end`
- `reply`
- `custom <arbitrary JSON>`

Example:

```yaml
routing:
  branch:
    on_status:
      ok: next
      retry: handler
    default: end
```

If a node yields no route (and no routing is declared), the flow run terminates. Flow-level semantics like “flow2flow” or “agent2agent” are implemented via components using generic host interfaces, **not** via special routing types.

---

### 4.4 Validation Rules

Core validations for `.ygtc`:

1. **Non-empty nodes**  
   - A flow must have at least one node.

2. **Implicit ingress**  
   - The first node in `nodes` is ingress and must exist.

3. **Routing targets**  
   - Routing variants that reference node IDs (`next`, `branch`) must reference nodes present in `nodes`.

4. **Component support**  
   - If a node references a component and the component manifest is available, the component’s `supports` must include this flow’s kind.

---

## 5. Component Specification

### 5.1 Component Manifest

Conceptual YAML structure:

```yaml
id: greentic.deploy.generic.iac
version: 1.0.0

supports:
  - events

world: "greentic:deploy-plan@1.0.0"

profiles:
  default: iac-generator
  supported:
    - iac-generator

capabilities:
  wasi:
    filesystem:
      mode: sandboxed         # none | readonly | sandboxed
      mounts:
        - name: iac-out
          host_class: iac_out # opaque host class key
          guest_path: /iac
    random: true
    clocks: true
    env:
      allow:
        - RUST_LOG
  host:
    secrets:
      required:
        - OAUTH_{PROVIDER}_CLIENT_ID  # example convention, not enforced by spec
    state:
      read: true
      write: false
    messaging:
      inbound: false
      outbound: false
    events:
      inbound: false
      outbound: true
    http:
      client: true
      server: false
    telemetry:
      scope: tenant
    oauth:
      client: true
    iac:
      write_templates: true
      execute_plans: false

configurators:
  basic: configure_generic_iac_generator
  full: configure_generic_iac_generator_full
```

Fields:

- `id` — logical component ID (string).
- `version` — semantic version.
- `supports` — flow kinds (e.g. `messaging`, `event`, `component_config`, `job`, `http`).
- `world` — WIT world name (string; exact mapping defined by `greentic-interfaces`).
- `profiles` — profile metadata.
- `capabilities` — detailed capability description.
- `operations` — list of operations with input/output schemas.
- `config_schema` — optional JSON schema for configuration.
- `resources` — resource hints (cpu/memory/latency).
- `configurators` — mapping to configurator flow IDs.

### 5.2 Capabilities

#### 5.2.1 WASI Capabilities

- `filesystem`:
  - `mode`: `none`, `readonly`, or `sandboxed`.
  - `mounts`: list of mounts:
    - `name`: logical name (e.g. `scratch`, `cache`, `iac-out`)
    - `host_class`: host-defined class key that maps to a real path
    - `guest_path`: path inside the guest (e.g. `/tmp`, `/iac`)
- `env`:
  - `allow`: environment variables allowed to be visible to the guest.
- `random` (bool)
- `clocks` (bool)

#### 5.2.2 Host Capabilities (Generic)

- `secrets`:
  - `required`: list of logical secret keys (e.g. `OPENAI_API_KEY`).

- `state`:
  - `read`: bool
  - `write`: bool

- `messaging`:
  - `inbound`: bool
  - `outbound`: bool

- `events`:
  - `inbound`: bool
  - `outbound`: bool

- `http`:
  - `client`: bool
  - `server`: bool

- `telemetry`:
  - `scope`: e.g. `tenant`, `pack`, `node`.

- `oauth` (NEW):
  - `client`: bool  
    Indicates that the component expects to use the OAuth broker host interface.

- `iac` (NEW):
  - `write_templates`: bool  
    Indicates the component may write infrastructure-as-code artifacts.
  - `execute_plans`: bool  
    Indicates the component may request plan execution (host decides how).

These capabilities are **purely generic**.

### 5.3 Profiles

Profiles are named capability bundles.

Example:

```yaml
profiles:
  default: stateless-agent
  supported:
    - stateless-agent
    - cache-agent
```

The exact mapping of profile name → capabilities is handled internally by the component and/or tooling; flows only see profile names.

### 5.4 Configurator Flows

Configurator flows are standard `kind: messaging` flows that:

1. Ask questions via cards/messages.
2. Collect answers.
3. Produce a `config` object for the component.

Example basic configurator:

```yaml
type: messaging
id: configure_generic_iac_generator
description: "Minimal config for the generic IaC generator"

nodes:
  start:
    message.card:
      config:
        card:
          title: "Configure IaC generator"
          fields:
            - id: output_format
              type: select
              label: "Output format"
              options:
                - "hcl"
                - "yaml"
                - "json"
      routing:
        default: build

  build:
    build.config:
      config:
        target_component: greentic.deploy.generic.iac
```

Tools:

- Run the configurator flow once.
- Capture the resulting config object.
- Insert it into the target flow’s node config.

### 5.5 WIT Worlds

Components import WIT worlds defined in `greentic-interfaces`, for example:

- `greentic:secrets/store@1.0.0`
- `greentic:state/store@1.0.0`
- `greentic:messaging/session@1.0.0`
- `greentic:events/emitter@1.0.0`
- `greentic:http/client@1.0.0`
- `greentic:telemetry/logger@1.0.0`
- `greentic:deploy-plan@1.0.0` (deployment flows)
- `greentic:oauth-broker@1.0.0` (OAuth flows)

The shared spec does not constrain the internal structure of these worlds beyond “generic host interactions”.

---

## 6. Pack Specification (`.gtpack`)

### 6.1 Directory Layout

Canonical layout:

```text
my-pack.gtpack/
  manifest.yaml
  flows/
    support_triage.ygtc
    webhook_to_http.ygtc
  components/           # optional; embedded components (fat packs)
    qa_agent.wasm
    deployer.wasm
  assets/               # optional; cards, prompts, icons…
    cards/
    prompts/
    icons/
```

Packs can be:

- **Thin** (recommended): flows + references to external components.  
- **Fat**: include embedded WASM modules for offline deployment.

### 6.2 Pack Manifest & `PackKind`

Example:

```yaml
id: greentic.example.support
version: 1.0.0
name: "Support Assistant Pack"

kind: application       # application | provider | infrastructure | library

flows:
  - id: support_triage
    kind: messaging
    tags: [support]
    entrypoints: ["default"]
    flow:
      schema_version: flow-v1
      id: support_triage
      kind: messaging
      entrypoints:
        default: {}
      nodes:
        ingress:
          id: ingress
          component:
            id: greentic.process.router
          input: { mapping: {} }
          output: { mapping: {} }
          routing:
            branch:
              on_status:
                billing: billing
                tech: tech
              default: fallback
          telemetry: {}
        billing:
          id: billing
          component:
            id: greentic.agent.qa
            operation: handle_billing
          input: { mapping: {} }
          output: { mapping: {} }
          routing: { end: {} }
          telemetry: {}
    entrypoints: ["default"]

components:
  - id: greentic.agent.qa
    version: "1.2.0"
    supports: [messaging]
    operations:
      - name: handle_billing
        input_schema: {}
        output_schema: {}
    resources: {}
  - id: greentic.process.router
    version: "0.9.0"
    supports: [messaging]
    operations:
      - name: route
        input_schema: {}
        output_schema: {}
    resources: {}

dependencies:
  - alias: provider.messaging
    pack_id: vendor.messaging.telegram
    version_req: "^1.0"
    required_capabilities: ["messaging"]

capabilities:
  - name: messaging
    description: "needs messaging surface"

signatures:
  signatures: []
```

Fields:

- `id` — logical pack ID.
- `version` — semantic version.
- `name` — human-friendly.
- `kind` — hint:
  - `application` — standard digital worker.
  - `provider` — component provider packs.
  - `infrastructure` — infrastructure packs.
  - `library` — shared building blocks.
- `flows[]` — embedded Flow entries (FlowKind + Flow).
- `components[]` — component manifests bundled in the pack.
- `dependencies[]` — pack dependencies with aliases and required capabilities.
- `capabilities` — capability declarations.
- `signatures` — detached signatures bundle.

### 6.3 Connectors

Connectors are optional, high-level mapping hints interpreted by hosts (runners/channel adapters/deployer). Flows remain connector-agnostic; embedded Flow definitions do not depend on connector configuration. Hosts may ignore connectors entirely or express ingress wiring elsewhere.

---

## 7. Deployment Extension

### 7.1 DeploymentPack vs ApplicationPack

The `kind` field distinguishes:

- `application` — normal digital worker packs.
- `deployment` — packs whose flows **primarily operate on DeploymentPlan** and produce IaC or provisioning effects (mapped to `infrastructure` in the new `PackKind` list).
- `mixed` — both roles (use `application` + suitable dependencies if needed).

This is a hint only; core logic does not require it.

### 7.2 DeploymentPlan

Provider-agnostic description of desired runtime topology.

Key fields (conceptual):

```yaml
pack_id: greentic.example.support
pack_version: 1.0.0

tenant: acme
environment: staging

runners:
  - name: core-runner
    replicas: 2
    capabilities: {}

messaging:
  logical_cluster: "core"
  subjects:
    - name: "support.inbound"
      purpose: "events"
      durable: true
      extra: {}

channels:
  - name: "teams-support"
    flow_id: support_triage
    kind: "teams"
    config: {}

secrets:
  - key: OPENAI_API_KEY
    required: true
    scope: "tenant"

oauth:
  - provider_id: "google"
    logical_client_id: "greentic-google-client"
    redirect_path: "/oauth/callback/google"
    extra: {}

telemetry:
  required: true
  suggested_endpoint: null
  extra: {}

extra: {}
```

Important:

- No provider-specific fields (no regions, VPCs, etc.)  
- `extra` is extension space  

### 7.3 Deployment Flows (Events Flows)

Deployment flows are **normal `kind: event` flows** that:

- Read `DeploymentPlan` via WIT `greentic:deploy-plan`.
- Generate IaC templates into preopened filesystems (e.g. `/iac`).
- Optionally emit status updates via `emit-status`.
- Optionally trigger plan execution if `iac.execute_plans` is enabled.

Example:

```yaml
type: events
id: deploy_generic_iac
description: "Generate IaC for the deployment plan"

nodes:
  render:
    deploy.renderer:
      component: greentic.deploy.generic.iac
      profile: iac-generator
      config: {}
      routing:
        default: done

  done:
    noop:
      config: {}
```

Ingress event payload might be:

```json
{ "deployment_plan_ref": "deploy-plan://current" }
```

But the real plan is provided via the WIT world.

### 7.4 WIT `greentic:deploy-plan@1.0.0`

World API (conceptual):

```wit
package greentic:deploy-plan@1.0.0;

world plan {
    /// Returns the current DeploymentPlan as a JSON string.
    get-deployment-plan: func() -> string

    /// Emit a generic status message about deployment progress.
    emit-status: func(message: string)
}
```

- JSON matches `DeploymentPlan` structure defined in `greentic-types`.
- No provider names or details in this interface.

### 7.5 IaC Capabilities

Host capability block:

```yaml
host:
  iac:
    write_templates: true
    execute_plans: false
```

Meaning:

- `write_templates: true` — component may write IaC artifacts to a host-managed mount (e.g., `/iac`).
- `execute_plans: true` — component may request execution of IaC plans (host decides how to implement).

The spec does not prescribe:

- Terraform vs CloudFormation vs Pulumi vs anything else.

---

## 8. OAuth Extension

### 8.1 Host Capability: OAuth

Components that need OAuth broker functionality set:

```yaml
host:
  oauth:
    client: true
```

This signals to:

- `greentic-pack` / `greentic-runner` that `greentic:oauth-broker` world should be wired.
- `greentic-oauth` to provide implementation.

### 8.2 WIT `greentic:oauth-broker@1.0.0`

Conceptual world:

```wit
package greentic:oauth-broker@1.0.0;

world broker {
    /// Build a consent URL for user redirection.
    get-consent-url: func(
        provider-id: string,
        subject: string,
        scopes: list<string>,
        redirect-path: string,
        extra-json: string,
    ) -> string

    /// Exchange an auth code for a token set.
    exchange-code: func(
        provider-id: string,
        subject: string,
        code: string,
        redirect-path: string,
    ) -> string

    /// Retrieve a stored token set for provider/subject/scopes.
    get-token: func(
        provider-id: string,
        subject: string,
        scopes: list<string>,
    ) -> string
}
```

Notes:

- `provider-id`: logical provider name (e.g. `"google"`, `"microsoft"`).
- `subject`: tenant- or user-level subject ID.
- `redirect-path`: path-only; host determines domain.
- `extra-json`: arbitrary JSON string for extra parameters.
- Returned strings are JSON-encoded token sets.

### 8.3 Token Semantics (Opaque)

Token sets are represented as JSON; fields are opaque to flows:

```json
{
  "access_token": "....",
  "refresh_token": "....",
  "expires_at": 1234567890,
  "token_type": "Bearer",
  "extra": { "id_token": "..." }
}
```

Only `greentic-oauth` and host logic interpret token structure.

---

## 9. Bindings Generation

Bindings are derived per node from:

- Component capabilities  
- Selected profile  
- Pack-level defaults  
- Tenant/team/user policies  
- Environment configuration (host-specific)  

### 9.1 Inputs

For each node:

- `Flow` (kind, nodes, routing)
- `ComponentManifest` (capabilities, profiles)
- Pack-level defaults
- `TenantCtx`
- Security policies and host config

### 9.2 Pipeline Steps

1. **Load pack** (manifest + flows).
2. **Resolve components** (from embedded components or registries).
3. **Compute effective capabilities**:
   - Determine profile (node → component → default).
   - Apply profile overlays.
4. **Apply environment policies**:
   - Map `host_class` to actual host paths.
   - Decide env vars allowed.
   - Decide network access.
5. **Generate WASI bindings**:
   - Preopens (paths + rights).
   - Env variables.
   - Stdout/stderr behaviour.
6. **Generate host bindings**:
   - Secrets, state, messaging, events, http, telemetry, oauth, iac.
   - For OAuth: ensure WIT world is wired if `host.oauth.client` is true.
   - For IaC: preopen `/iac` or equivalent if `host.iac.write_templates` is true.
7. **Instantiate WASM**:
   - Use Wasmtime with WASI Preview 2.
   - Inject WIT host worlds.

### 9.3 Debug/Inspection

Tooling like `greentic-pack` SHOULD provide:

- `pack capabilities <pack>` — show effective capabilities per node.
- `pack bindings <pack> --mode strict|complete` — show generated bindings:
  - `strict`: exactly as declared.
  - `complete`: allow defaults (e.g., scratch mount, minimal env).

---

## 10. Execution Model

### 10.1 Messaging Flows

1. External connector (Teams, WebChat, etc.) receives a message.
2. Runner resolves which flow to invoke.
3. First node of the flow receives a `MessagingEnvelope`.
4. Node executes in WASM sandbox:
   - Reads input payload.
   - Produces new payload + route label/next node.
5. Routing chooses next node.
6. Process repeats until:
   - A reply node sends a response.
   - No further routing (terminal).
   - A component explicitly ends the flow.

### 10.2 Events Flows

1. External trigger (webhook, queue, timer, deployment request).
2. Runner invokes the flow with an event payload.
3. First node processes event.
4. Nodes produce side effects (HTTP, events, files, etc.).
5. Flow terminates when no further routing.

### 10.3 Deployment Flows

Same as events flows, with additional capabilities:

- Access `DeploymentPlan` via `greentic:deploy-plan` world.
- Write IaC artifacts to `/iac` (or equivalent).
- Emit status updates (via telemetry and/or `emit-status`).
- Optionally request plan execution if `execute_plans` is enabled.

---

## 11. Tool Responsibilities

### 11.1 `greentic-pack`

- Owns pack/flow/component parsing and validation.
- Aggregates capabilities and profiles.
- Generates bindings (debug/preview).
- Infers base `DeploymentPlan` from pack + tenant/environment (`pack plan`).

### 11.2 `greentic-runner`

- Loads packs and generated bindings.
- Instantiates WASM with Wasmtime P2.
- Wires all host worlds.
- Executes messaging/events/deployment flows.

### 11.3 `greentic-oauth`

- Implements `greentic:oauth-broker` world.
- Manages OAuth clients and token storage.
- Integrates with secrets/config/TenantCtx.
- Optionally processes `DeploymentPlan.oauth` entries.

### 11.4 `greentic-deployer`

- Owner of `DeploymentPlan` lifecycle.
- Builds plans for application packs.
- Maps provider+strategy → deployment packs/flows.
- Runs deployment flows via `greentic-runner`.
- Manages IaC artifacts and apply/destroy operations.

### 11.5 `greentic-dev`

- Scaffolds packs/flows/components.
- Runs configurator flows.
- Provides local execution and linting.

### 11.6 `greentic-demo`

- Provides canonical example packs (application + deployment).
- Demonstrates best practices.

### 11.7 `greentic-conformance` & `greentic-integration`

- Conformance: checks adherence to this spec (shape + behaviour).
- Integration: cross-repo tests (pack → bindings → runner → host worlds).

---

## 12. Conformance Expectations

A repo/tool is **conformant** if:

- It treats IDs/kinds as opaque strings.
- It uses `greentic-types` models (or faithfully equivalent) for:
  - Flow
  - PackManifest
  - ComponentManifest
  - DeploymentPlan
- It does not embed provider-specific semantics in:
  - Shared types
  - WIT interface definitions
- It uses bindings generation rather than manual bindings.
- It uses WIT worlds as defined in `greentic-interfaces`.

Levels (informal):

- **Level 1:** Understands flows, components, packs (no deployment/OAuth).  
- **Level 2:** Adds DeploymentPlan + deploy-plan world support.  
- **Level 3:** Adds OAuth broker support and uses OAuthPlan from DeploymentPlan.  

---

## 13. Glossary

- **Flow** — `.ygtc` YAML file describing a graph of nodes with routing.
- **Node** — a single component invocation step in a flow.
- **Component** — WASM module + manifest (capabilities, profiles, configurators).
- **Pack** — `.gtpack` directory/archive with manifest + flows (+ optional components/assets).
- **Bindings** — runtime configuration for WASI + host worlds, derived automatically.
- **TenantCtx** — tenant/team/user context for multi-tenant execution.
- **DeploymentPlan** — provider-agnostic description of desired deployment topology.
- **Configurator Flow** — messaging flow used to configure a component.
- **Messaging Flow** — `kind: messaging` flow triggered by conversational channels.
- **Event Flow** — `kind: event` flow triggered by arbitrary events (webhook, timers, deployment).
- **Deployment Flow** — an event flow that operates on a `DeploymentPlan`.
- **Host Capability** — a generic host service (secrets, state, messaging, events, http, telemetry, oauth, iac).
- **WIT World** — interface definition for WASM host/guest interaction.

---

## 14. Worked Examples

### 14.1 Messaging Flow — Support Triage (Flow model)

```yaml
schema_version: flow-v1
id: support_triage
kind: messaging
entrypoints:
  default: {}
nodes:
  router:
    id: router
    component:
      id: greentic.process.router
      operation: route
    input: { mapping: {} }
    output: { mapping: {} }
    routing:
      branch:
        on_status:
          billing: billing
          qa: qa
        default: fallback
    telemetry: {}

  billing:
    id: billing
    component:
      id: greentic.agent.qa
      operation: handle_billing
    input: { mapping: {} }
    output: { mapping: {} }
    routing:
      end: {}
    telemetry: {}

  qa:
    id: qa
    component:
      id: greentic.agent.qa
      operation: handle_qa
    input: { mapping: {} }
    output: { mapping: {} }
    routing:
      end: {}
    telemetry: {}

  fallback:
    id: fallback
    component:
      id: greentic.messaging.reply
      operation: reply
    input:
      mapping:
        template: { text: "I will transfer you to a human agent." }
    output: { mapping: {} }
    routing:
      end: {}
    telemetry: {}
```

### 14.2 Event Flow — Webhook → Transform → HTTP (Flow model)

```yaml
schema_version: flow-v1
id: webhook_to_http
kind: event
entrypoints:
  webhook: {}
nodes:
  ingress:
    id: ingress
    component:
      id: greentic.webhook.ingress
      operation: receive
    input: { mapping: {} }
    output: { mapping: {} }
    routing:
      next:
        node_id: transform
    telemetry: {}

  transform:
    id: transform
    component:
      id: greentic.process.transform
      operation: render
    input:
      mapping:
        template: |
          {
            "id": "{{payload.id}}",
            "value": "{{payload.value}}"
          }
    output: { mapping: {} }
    routing:
      next:
        node_id: http_out
    telemetry: {}

  http_out:
    id: http_out
    component:
      id: greentic.http.forwarder
      operation: send
    input:
      mapping:
        method: POST
        url: https://api.partner.com/orders
    output: { mapping: {} }
    routing:
      end: {}
    telemetry: {}
```

### 14.3 Deployment Flow — Generic IaC

```yaml
type: events
id: deploy_generic_iac
description: Generate IaC from DeploymentPlan and write to /iac

nodes:
  render:
    deploy.renderer:
      component: greentic.deploy.generic.iac
      profile: iac-generator
      config: {}
      routing:
        default: done

  done:
    noop:
      config: {}
```

### 14.4 OAuth Usage — Conceptual Component Behaviour

A component that needs an access token might:

1. Call `get-token(provider-id, subject, scopes)` via `greentic:oauth-broker`.
2. If no token is found, it might:
   - Call `get-consent-url` and send it to the user via messaging.
   - Wait for an auth code (out-of-band).
   - Call `exchange-code` to obtain and store tokens.
3. Use the access token in HTTP calls (via `greentic:http/client`).

The flow does not change for OAuth-specific behaviour; only the component logic and host worlds do.

---

**End of Specification**
