#![cfg(all(feature = "serde", feature = "time"))]

use greentic_types::{
    AttestationStatement, BuildPlan, BuildStatus, BuildStatusKind, MetadataRecord, PredicateType,
    RegistryRef, RepoContext, ScanKind, ScanRequest, ScanResult, ScanStatusKind, SignRequest,
    StoreContext, StoreRef, VerifyRequest, VerifyResult,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::json;
use time::OffsetDateTime;
use time::macros::datetime;

fn assert_roundtrip<T>(value: &T)
where
    T: Serialize + DeserializeOwned + PartialEq + core::fmt::Debug,
{
    let json = serde_json::to_string_pretty(value).expect("serialize");
    let roundtrip: T = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(&roundtrip, value, "{json}");
}

#[test]
fn build_plan_and_status_roundtrip() {
    let mut plan = BuildPlan {
        build_id: "build-1".parse().unwrap(),
        component: "component.repo".parse().unwrap(),
        branch: None,
        source_repo: "repo-main".parse().unwrap(),
        commit: "deadbeef".into(),
        commit_ref: None,
        language: "rust".into(),
        entrypoint: "cargo build".into(),
        env: Default::default(),
        outputs: vec!["artifact-1".parse().unwrap()],
        metadata: json!({"target": "x86_64-unknown-linux-gnu"}),
    };
    plan.env.insert("RUSTFLAGS".into(), "-Dwarnings".into());

    assert_roundtrip(&plan);

    let status = BuildStatus {
        build_id: plan.build_id.clone(),
        status: BuildStatusKind::Succeeded,
        started_at_utc: Some(datetime!(2025-01-02 03:04:05 UTC)),
        finished_at_utc: Some(datetime!(2025-01-02 03:14:05 UTC)),
        artifacts: plan.outputs.clone(),
        logs_ref: Some("logs://build-1".into()),
        log_refs: vec!["log-1".parse().unwrap()],
        metadata: json!({"duration_ms": 600000}),
    };

    assert_roundtrip(&status);
}

#[test]
fn scan_request_and_result_roundtrip() {
    let request = ScanRequest {
        scan_id: "scan-1".parse().unwrap(),
        component: "component.repo".parse().unwrap(),
        kind: ScanKind::Dependencies,
        commit_ref: None,
        artifact: Some("artifact-1".parse().unwrap()),
        metadata: json!({"severity_threshold": "high"}),
    };

    assert_roundtrip(&request);

    let result = ScanResult {
        scan_id: request.scan_id.clone(),
        component: request.component.clone(),
        kind: request.kind.clone(),
        status: ScanStatusKind::Succeeded,
        sbom: Some("sbom-1".parse().unwrap()),
        findings: json!({"vulns": [{"id": "CVE-1234"}]}),
        started_at_utc: Some(OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap()),
        finished_at_utc: Some(OffsetDateTime::from_unix_timestamp(1_700_000_500).unwrap()),
    };

    assert_roundtrip(&result);
}

#[test]
fn signing_and_verification_roundtrip() {
    let sign_request = SignRequest {
        signing_key: "key-1".parse().unwrap(),
        artifact: "artifact-2".parse().unwrap(),
        payload: json!({"digest": "sha256:deadbeef"}),
        metadata: json!({"purpose": "release"}),
    };

    assert_roundtrip(&sign_request);

    let verify_request = VerifyRequest {
        signature: "sig-1".parse().unwrap(),
        artifact: sign_request.artifact.clone(),
        metadata: json!({"policy": "default"}),
    };

    assert_roundtrip(&verify_request);

    let verify_result = VerifyResult {
        signature: verify_request.signature.clone(),
        valid: true,
        message: Some("signature matches".into()),
        metadata: json!({"checked_at": "2025-01-02T03:04:05Z"}),
    };

    assert_roundtrip(&verify_result);
}

#[test]
fn attestation_and_metadata_roundtrip() {
    let attestation = AttestationStatement {
        attestation_id: Some("att-1".parse().unwrap()),
        attestation: "att-1".parse().unwrap(),
        predicate_type: PredicateType::Slsa,
        statement: "stmt-1".parse().unwrap(),
        registry: Some("registry-1".parse::<RegistryRef>().unwrap()),
        store: Some("store-1".parse::<StoreRef>().unwrap()),
        metadata: json!({"builder": "slsa-generator"}),
    };

    assert_roundtrip(&attestation);

    let record = MetadataRecord {
        version: Some("v1".parse().unwrap()),
        namespace: Some("scan.snyk".into()),
        key: "cvss_max".into(),
        value: json!(9.5),
    };

    assert_roundtrip(&record);
}

#[test]
fn context_wrappers_roundtrip() {
    let tenant =
        greentic_types::TenantCtx::new("prod".parse().unwrap(), "tenant-1".parse().unwrap());
    let repo_ctx = RepoContext {
        tenant: tenant.clone(),
        repo: "repo-main".parse().unwrap(),
    };
    let store_ctx = StoreContext {
        tenant,
        store: "store-primary".parse().unwrap(),
    };

    assert_roundtrip(&repo_ctx);
    assert_roundtrip(&store_ctx);
}
