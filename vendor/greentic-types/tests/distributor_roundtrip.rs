#![cfg(feature = "serde")]

use greentic_types::{
    ArtifactLocation, CacheInfo, ComponentDigest, ComponentStatus, DistributorEnvironmentId,
    ResolveComponentRequest, ResolveComponentResponse, SignatureSummary, TenantCtx, TenantId,
};
use serde_json::json;

fn sample_ctx() -> TenantCtx {
    let env = "prod".parse().unwrap();
    let tenant: TenantId = "tenant-1".parse().unwrap();
    TenantCtx::new(env, tenant)
}

#[test]
fn resolve_component_request_roundtrip() {
    let req = ResolveComponentRequest {
        tenant: sample_ctx(),
        environment_id: DistributorEnvironmentId::from("env-1"),
        pack_id: "pack.alpha".into(),
        component_id: "component.beta".into(),
        version: "1.2.3".into(),
        extra: json!({"hint": "warm"}),
    };

    let json = serde_json::to_string_pretty(&req).unwrap();
    let roundtrip: ResolveComponentRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(req, roundtrip);
}

#[test]
fn resolve_component_response_roundtrip() {
    let resp = ResolveComponentResponse {
        status: ComponentStatus::Ready,
        digest: ComponentDigest::from(
            "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        ),
        artifact: ArtifactLocation::FilePath {
            path: "/tmp/component.wasm".into(),
        },
        signature: SignatureSummary {
            verified: true,
            signer: "sig-key-1".into(),
            extra: json!({"note": "dev signature"}),
        },
        cache: CacheInfo {
            size_bytes: 42,
            last_used_utc: "2025-01-01T00:00:00Z".into(),
            last_refreshed_utc: "2025-01-01T00:00:00Z".into(),
        },
    };

    let json = serde_json::to_string_pretty(&resp).unwrap();
    let roundtrip: ResolveComponentResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(resp, roundtrip);
}

#[test]
fn component_digest_sha256_like_check() {
    let ok = ComponentDigest::from(
        "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    );
    assert!(ok.is_sha256_like());

    for bad in [
        "sha256:short",
        "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdeg",
        "sha512:0123",
        "not-a-digest",
    ] {
        let digest = ComponentDigest::from(bad);
        assert!(!digest.is_sha256_like(), "digest {bad} should be rejected");
    }
}
