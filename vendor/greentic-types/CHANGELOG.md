# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]
- Added domain-agnostic worker envelope models (`WorkerRequest`, `WorkerMessage`, `WorkerResponse`) mirroring `greentic:worker@1.0.0`, with serde/schemars derives, schema exports, tests, and docs.
- Added generic `EventEnvelope` + `EventId`, provider capability descriptors (`EventProviderDescriptor` with `EventProviderKind`, transports, reliability, ordering), and shared `ChannelMessageEnvelope` + `Attachment` for channel messaging; all derive serde/schemars and export JSON Schemas.
- Added supply-chain shared types and ID newtypes (RepoRef, ComponentRef, BuildRef, ScanRef, AttestationRef, PolicyRef, StoreRef, RegistryRef) plus BuildPlan/BuildStatus, ScanRequest/ScanResult, signing/verification refs, attestation statements, metadata records, and contexts, with schema exports and serde round-trip tests.
- Added store/distributor models and IDs (StoreFrontId, StoreProductId, StorePlanId, SubscriptionId, EnvironmentRef, DistributorRef, BundleId, CollectionId, MetadataRecordRef) and schemas.
- Storefront/cms models: Theme, LayoutSection, Collection, ProductOverride, StoreFront with serde/schemars and tests.
- Catalog/pricing models: StoreProduct (+CapabilityMap, VersionStrategy), StorePlan (+PriceModel, PlanLimits), Subscription (+status), DesiredState (+entries), BundleSpec, DesiredStateExportSpec, Environment (+ConnectionKind); wired into schema exports and round-trip tests.
- Extended shared models with tenant attributes, supply-chain and policy/distribution refs (branch/commit/webhook/version/attestation/policy-input/oci/build-log), Environment name + helper, RolloutStatus, CapabilityMap as open map, VersionStrategy tagged+legacy serde, PolicyDecision status/reasons, optional attestation IDs, and updated schema exports + round-trip tests.
- Added `PackKind` to `PackManifest`, optional `host.iac` capabilities for components, and the generic `DeploymentPlan` family shared across runner/deployer repositories.
- Added the schema exporter binary, GitHub Pages workflow, and [`SCHEMAS.md`](SCHEMAS.md) so IDEs/CLIs can validate documents against canonical `$id`s.
- Documented feature flags + MSRV (Rust 1.85), introduced the `schema`/`otel-keys` flags, and exposed the crate `VERSION` constant.
- Hardened ID newtypes and `SemverReq` with `FromStr`/`TryFrom` implementations, serde guards, and property tests ensuring invalid identifiers cannot deserialize.
- Added CI checks for duplicate struct definitions and public schema `$id` sanity.
- Added `pack_spec` module with canonical `PackSpec` and `ToolSpec` structures for `pack.yaml` parsing.
- Introduced shared deployment context primitives (`Cloud`, `Platform`, `DeploymentCtx`) and made them available via `greentic_types`.
- Added generic `.ygtc` flow models, component manifests, and `.gtpack` manifest types (plus schema exports) under `flow`, `component`, and `pack_manifest`.
- Exposed helper APIs like `Flow::ingress`, `Flow::validate_components`, and `ComponentManifest::select_profile` to keep validation/profile logic consistent across repos.
- Added `MODELS.md` + README guidance describing the opaque-ID/capabilities-only design and marked `pack_spec` as legacy for migration planning.
- Removed deprecated `pack_spec` module now that `.gtpack` manifests are canonical.

## [0.4.9]
- Add provider ID newtypes for shared usage: `GitProviderRef` and `ScannerRef`, with schema exports and serde round-trip tests.

## [0.1.0] - 2025-10-23
- Initial release with tenant identifiers, context, and invocation envelope
- Added `NodeError`/`NodeResult` with retry and detail helpers
- Added idempotency key and safe JSON helpers with unit tests
