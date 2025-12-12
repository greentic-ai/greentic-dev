# Greentic Model Guidelines

Greentic packs, flows, and components deliberately keep semantics opaque so the same types apply to
every domain. The models exported by `greentic-types` follow three simple rules:

1. **Identifiers are untyped strings.**  
   `PackId`, `FlowId`, `NodeId`, `ComponentId`, node kinds, and profile names only model identity. A
   runtime may interpret `agent.router` or `component-a`, but the type system treats them as strings
   without baked-in categories.

2. **Capabilities describe interaction patterns, not intent.**  
   `ComponentCapabilities` only list WASI surfaces (`filesystem`, `env`, `random`, `clocks`) and
   host services (`secrets`, `state`, `messaging`, `events`, `http`, `telemetry`). Nothing in the
   manifest says “QA agent” or “RAG caller”; the runtime just wires the declared interfaces and
   policies.

3. **Packs and flows never encode bindings or connectors.**  
   `Flow` holds a DAG of nodes with explicit routing (`Routing` enum) and opaque config/mapping
   blobs. `PackManifest` embeds flows directly alongside component manifests, dependencies, and
   capabilities, but no WASI paths, secrets, or tenant-specific wiring. Bindings are generated
   later by hosts like `greentic-pack` and `greentic-runner`.

This separation lets humans and small LLMs edit `.ygtc` and `.gtpack` files confidently while hosts
handle sandboxing, policies, and domain-specific behaviour elsewhere.

## Pack kinds

`PackManifest` carries a `kind` hint (`application`, `provider`, `infrastructure`, `library`). It is
advisory; runtimes still treat flows uniformly. The hint gives CLIs and UX a way to highlight packs
without forcing new flow kinds.

## Host IaC capabilities

`HostCapabilities` exposes a generic `iac` block so components can declare whether they write
infrastructure-as-code artifacts and/or trigger plan execution. The flags describe access to a
preopened filesystem area and an optional “execute plans” hook; no provider names or tool-specific
semantics appear in `greentic-types`.

## Deployment plans

`DeploymentPlan` is a provider-neutral shape passed between `greentic-pack`, `greentic-runner`, and
`greentic-deployer`. It lists the pack identity, tenant/environment context, runners, messaging,
channels, secrets, OAuth clients, telemetry hints, and an `extra` blob for future fields. Nothing in
the type references regions, clusters, or other provider details—deployment components interpret the
plan and translate it into concrete infrastructure.
