# JSON Schema Publishing

Schemas for the shared Greentic types are published to GitHub Pages so IDEs, CLIs, and CI jobs can validate documents consistently. Every schema lives under the stable base URL:

```
https://greentic-ai.github.io/greentic-types/schemas/v1/<name>.schema.json
```

The `bin/export-schemas.rs` helper (or `greentic_types::write_all_schemas`) materialises the schemas into `dist/schemas/v1/`. The GitHub Pages workflow runs the helper on every push to `master` and republishes the `dist/` directory.

The v1 flow/pack model now embeds flows inside `PackManifest` and publishes the Flow schema as `greentic.flow.v1` and the pack schema as `greentic.pack-manifest.v1`.

## Canonical URLs

| Type | Schema URL |
|------|------------|
| PackId | https://greentic-ai.github.io/greentic-types/schemas/v1/pack-id.schema.json |
| ComponentId | https://greentic-ai.github.io/greentic-types/schemas/v1/component-id.schema.json |
| FlowId | https://greentic-ai.github.io/greentic-types/schemas/v1/flow-id.schema.json |
| NodeId | https://greentic-ai.github.io/greentic-types/schemas/v1/node-id.schema.json |
| TenantContext | https://greentic-ai.github.io/greentic-types/schemas/v1/tenant-context.schema.json |
| HashDigest | https://greentic-ai.github.io/greentic-types/schemas/v1/hash-digest.schema.json |
| SemverReq | https://greentic-ai.github.io/greentic-types/schemas/v1/semver-req.schema.json |
| RedactionPath | https://greentic-ai.github.io/greentic-types/schemas/v1/redaction-path.schema.json |
| Capabilities | https://greentic-ai.github.io/greentic-types/schemas/v1/capabilities.schema.json |
| RepoSkin | https://greentic-ai.github.io/greentic-types/schemas/v1/repo-skin.schema.json |
| RepoAuth | https://greentic-ai.github.io/greentic-types/schemas/v1/repo-auth.schema.json |
| RepoTenantConfig | https://greentic-ai.github.io/greentic-types/schemas/v1/repo-tenant-config.schema.json |
| TenantDidDocument | https://greentic-ai.github.io/greentic-types/schemas/v1/tenant-did-document.schema.json |
| Flow (greentic.flow.v1) | https://greentic-ai.github.io/greentic-types/schemas/v1/flow.schema.json |
| Node | https://greentic-ai.github.io/greentic-types/schemas/v1/node.schema.json |
| ComponentManifest | https://greentic-ai.github.io/greentic-types/schemas/v1/component-manifest.schema.json |
| PackManifest (greentic.pack-manifest.v1) | https://greentic-ai.github.io/greentic-types/schemas/v1/pack-manifest.schema.json |
| Limits | https://greentic-ai.github.io/greentic-types/schemas/v1/limits.schema.json |
| TelemetrySpec | https://greentic-ai.github.io/greentic-types/schemas/v1/telemetry-spec.schema.json |
| NodeSummary | https://greentic-ai.github.io/greentic-types/schemas/v1/node-summary.schema.json |
| NodeFailure | https://greentic-ai.github.io/greentic-types/schemas/v1/node-failure.schema.json |
| NodeStatus | https://greentic-ai.github.io/greentic-types/schemas/v1/node-status.schema.json |
| RunStatus | https://greentic-ai.github.io/greentic-types/schemas/v1/run-status.schema.json |
| TranscriptOffset | https://greentic-ai.github.io/greentic-types/schemas/v1/transcript-offset.schema.json |
| ToolsCaps | https://greentic-ai.github.io/greentic-types/schemas/v1/tools-caps.schema.json |
| SecretsCaps | https://greentic-ai.github.io/greentic-types/schemas/v1/secrets-caps.schema.json |
| BranchRef | https://greentic-ai.github.io/greentic-types/schemas/v1/branch-ref.schema.json |
| CommitRef | https://greentic-ai.github.io/greentic-types/schemas/v1/commit-ref.schema.json |
| GitProviderRef | https://greentic-ai.github.io/greentic-types/schemas/v1/git-provider-ref.schema.json |
| ScannerRef | https://greentic-ai.github.io/greentic-types/schemas/v1/scanner-ref.schema.json |
| WebhookId | https://greentic-ai.github.io/greentic-types/schemas/v1/webhook-id.schema.json |
| RepoRef | https://greentic-ai.github.io/greentic-types/schemas/v1/repo-ref.schema.json |
| ComponentRef | https://greentic-ai.github.io/greentic-types/schemas/v1/component-ref.schema.json |
| VersionRef | https://greentic-ai.github.io/greentic-types/schemas/v1/version-ref.schema.json |
| BuildRef | https://greentic-ai.github.io/greentic-types/schemas/v1/build-ref.schema.json |
| ScanRef | https://greentic-ai.github.io/greentic-types/schemas/v1/scan-ref.schema.json |
| AttestationRef | https://greentic-ai.github.io/greentic-types/schemas/v1/attestation-ref.schema.json |
| AttestationId | https://greentic-ai.github.io/greentic-types/schemas/v1/attestation-id.schema.json |
| PolicyRef | https://greentic-ai.github.io/greentic-types/schemas/v1/policy-ref.schema.json |
| PolicyInputRef | https://greentic-ai.github.io/greentic-types/schemas/v1/policy-input-ref.schema.json |
| StoreRef | https://greentic-ai.github.io/greentic-types/schemas/v1/store-ref.schema.json |
| RegistryRef | https://greentic-ai.github.io/greentic-types/schemas/v1/registry-ref.schema.json |
| OciImageRef | https://greentic-ai.github.io/greentic-types/schemas/v1/oci-image-ref.schema.json |
| ArtifactRef | https://greentic-ai.github.io/greentic-types/schemas/v1/artifact-ref.schema.json |
| SbomRef | https://greentic-ai.github.io/greentic-types/schemas/v1/sbom-ref.schema.json |
| SigningKeyRef | https://greentic-ai.github.io/greentic-types/schemas/v1/signing-key-ref.schema.json |
| SignatureRef | https://greentic-ai.github.io/greentic-types/schemas/v1/signature-ref.schema.json |
| StatementRef | https://greentic-ai.github.io/greentic-types/schemas/v1/statement-ref.schema.json |
| BuildLogRef | https://greentic-ai.github.io/greentic-types/schemas/v1/build-log-ref.schema.json |
| ApiKeyRef | https://greentic-ai.github.io/greentic-types/schemas/v1/api-key-ref.schema.json |
| BuildPlan | https://greentic-ai.github.io/greentic-types/schemas/v1/build-plan.schema.json |
| BuildStatus | https://greentic-ai.github.io/greentic-types/schemas/v1/build-status.schema.json |
| ScanRequest | https://greentic-ai.github.io/greentic-types/schemas/v1/scan-request.schema.json |
| ScanResult | https://greentic-ai.github.io/greentic-types/schemas/v1/scan-result.schema.json |
| SignRequest | https://greentic-ai.github.io/greentic-types/schemas/v1/sign-request.schema.json |
| VerifyRequest | https://greentic-ai.github.io/greentic-types/schemas/v1/verify-request.schema.json |
| VerifyResult | https://greentic-ai.github.io/greentic-types/schemas/v1/verify-result.schema.json |
| AttestationStatement | https://greentic-ai.github.io/greentic-types/schemas/v1/attestation-statement.schema.json |
| MetadataRecord | https://greentic-ai.github.io/greentic-types/schemas/v1/metadata-record.schema.json |
| RepoContext | https://greentic-ai.github.io/greentic-types/schemas/v1/repo-context.schema.json |
| StoreContext | https://greentic-ai.github.io/greentic-types/schemas/v1/store-context.schema.json |
| Bundle | https://greentic-ai.github.io/greentic-types/schemas/v1/bundle.schema.json |
| DesiredStateExportSpec | https://greentic-ai.github.io/greentic-types/schemas/v1/desired-state-export.schema.json |
| DesiredState | https://greentic-ai.github.io/greentic-types/schemas/v1/desired-state.schema.json |
| DesiredSubscriptionEntry | https://greentic-ai.github.io/greentic-types/schemas/v1/desired-subscription-entry.schema.json |
| ArtifactSelector | https://greentic-ai.github.io/greentic-types/schemas/v1/artifact-selector.schema.json |
| StoreFront | https://greentic-ai.github.io/greentic-types/schemas/v1/storefront.schema.json |
| StoreProduct | https://greentic-ai.github.io/greentic-types/schemas/v1/store-product.schema.json |
| StorePlan | https://greentic-ai.github.io/greentic-types/schemas/v1/store-plan.schema.json |
| CapabilityMap | https://greentic-ai.github.io/greentic-types/schemas/v1/capability-map.schema.json |
| Subscription | https://greentic-ai.github.io/greentic-types/schemas/v1/subscription.schema.json |
| Environment | https://greentic-ai.github.io/greentic-types/schemas/v1/environment.schema.json |
| RolloutStatus | https://greentic-ai.github.io/greentic-types/schemas/v1/rollout-status.schema.json |
| Theme | https://greentic-ai.github.io/greentic-types/schemas/v1/theme.schema.json |
| LayoutSection | https://greentic-ai.github.io/greentic-types/schemas/v1/layout-section.schema.json |
| Collection | https://greentic-ai.github.io/greentic-types/schemas/v1/collection.schema.json |
| ProductOverride | https://greentic-ai.github.io/greentic-types/schemas/v1/product-override.schema.json |
| StoreProductKind | https://greentic-ai.github.io/greentic-types/schemas/v1/store-product-kind.schema.json |
| VersionStrategy | https://greentic-ai.github.io/greentic-types/schemas/v1/version-strategy.schema.json |
| ConnectionKind | https://greentic-ai.github.io/greentic-types/schemas/v1/connection-kind.schema.json |
| PackOrComponentRef | https://greentic-ai.github.io/greentic-types/schemas/v1/pack-or-component-ref.schema.json |
| PlanLimits | https://greentic-ai.github.io/greentic-types/schemas/v1/plan-limits.schema.json |
| PriceModel | https://greentic-ai.github.io/greentic-types/schemas/v1/price-model.schema.json |
| SubscriptionStatus | https://greentic-ai.github.io/greentic-types/schemas/v1/subscription-status.schema.json |
| EventEnvelope | https://greentic-ai.github.io/greentic-types/schemas/v1/event-envelope.schema.json |
| EventProviderDescriptor | https://greentic-ai.github.io/greentic-types/schemas/v1/event-provider-descriptor.schema.json |
| ChannelMessageEnvelope | https://greentic-ai.github.io/greentic-types/schemas/v1/channel-message-envelope.schema.json |
| Attachment | https://greentic-ai.github.io/greentic-types/schemas/v1/attachment.schema.json |
| WorkerRequest | https://greentic-ai.github.io/greentic-types/schemas/v1/worker-request.schema.json |
| WorkerMessage | https://greentic-ai.github.io/greentic-types/schemas/v1/worker-message.schema.json |
| WorkerResponse | https://greentic-ai.github.io/greentic-types/schemas/v1/worker-response.schema.json |
| OtlpKeys | https://greentic-ai.github.io/greentic-types/schemas/v1/otlp-keys.schema.json |
| RunResult | https://greentic-ai.github.io/greentic-types/schemas/v1/run-result.schema.json |

> `OtlpKeys` and `RunResult` schemas are emitted when the `otel-keys` and `time` features are enabled respectively; both keep their canonical IDs.

## Generating locally

```bash
cargo run --bin export-schemas --all-features
ls dist/schemas/v1
```

Use these URLs in IDE validation rules, manifests, or CI assertions so other repos stay in sync with the shared types.
