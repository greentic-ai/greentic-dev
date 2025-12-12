#![cfg(feature = "serde")]

use greentic_types::{
    AllowList, Capabilities, ComponentId, ErrorCode, FsCaps, GitProviderRef, GreenticError,
    HashDigest, HttpCaps, Impersonation, InvocationDeadline, KvCaps, Limits, NetCaps,
    NetworkPolicy, NodeFailure, NodeId, NodeStatus, NodeSummary, Outcome, PackId, PackRef,
    PolicyDecision, PolicyDecisionStatus, RedactionPath, RunStatus, ScannerRef, SecretsCaps,
    SemverReq, SessionCursor, SessionKey, Signature, SignatureAlgorithm, SpanContext, StateKey,
    StatePath, TelemetrySpec, TenantContext, TenantCtx, TenantIdentity, ToolsCaps,
    TranscriptOffset,
};
#[cfg(feature = "time")]
use greentic_types::{FlowId, RunResult};
use semver::Version;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::BTreeMap;
use std::str::FromStr;

#[cfg(feature = "time")]
use time::{Duration, OffsetDateTime};

fn assert_roundtrip<T>(value: &T)
where
    T: Serialize + DeserializeOwned + PartialEq + core::fmt::Debug,
{
    let json = serde_json::to_string_pretty(value).expect("serialize");
    let roundtrip: T = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(&roundtrip, value, "{json}");
}

#[test]
fn tenant_ctx_roundtrip() {
    let mut ctx = TenantCtx::new("prod".parse().unwrap(), "tenant-1".parse().unwrap())
        .with_team(Some("team-9".parse().unwrap()))
        .with_user(Some("user-42".parse().unwrap()));
    ctx.trace_id = Some("trace-1".into());
    ctx.correlation_id = Some("corr-7".into());
    ctx.idempotency_key = Some("idem-3".into());
    ctx.deadline = Some(InvocationDeadline::from_unix_millis(42));
    ctx.impersonation = Some(Impersonation {
        actor_id: "support-ops".parse().unwrap(),
        reason: Some("break-glass".into()),
    });

    assert_roundtrip(&ctx);

    let identity = TenantIdentity::from(&ctx);
    assert_eq!(identity.tenant_id.as_str(), "tenant-1");
    assert_roundtrip(&identity);
}

#[test]
fn session_types_roundtrip() {
    let key = SessionKey::from("sess-123");
    let cursor = SessionCursor::new("node.entry")
        .with_wait_reason("awaiting-input")
        .with_outbox_marker("outbox-1");

    assert_roundtrip(&key);
    assert_roundtrip(&cursor);
}

#[test]
fn state_types_roundtrip() {
    let key = StateKey::from("state::demo");
    let mut path = StatePath::root();
    path.push("meta");
    path.push("progress");

    assert_roundtrip(&key);
    assert_roundtrip(&path);
    assert_eq!(path.to_pointer(), "/meta/progress");
    let parsed = StatePath::from_pointer("/meta/progress");
    assert_eq!(parsed, path);
}

#[test]
fn outcome_roundtrip() {
    let done: Outcome<String> = Outcome::Done("ok".into());
    let pending: Outcome<String> = Outcome::Pending {
        reason: "waiting".into(),
        expected_input: Some(vec!["user_input".into()]),
    };
    let error = Outcome::<String>::Error {
        code: ErrorCode::InvalidInput,
        message: "bad".into(),
    };

    assert_roundtrip(&done);
    assert_roundtrip(&pending);
    assert_roundtrip(&error);
}

#[test]
fn policy_roundtrip() {
    let list = AllowList {
        domains: vec!["api.greentic.ai".into()],
        ports: vec![443],
        protocols: vec![greentic_types::Protocol::Https],
    };

    let policy = NetworkPolicy {
        egress: list,
        deny_on_miss: true,
    };

    let decision = PolicyDecision {
        status: PolicyDecisionStatus::Allow,
        reasons: vec!["matched allow list".into()],
        allow: Some(true),
        reason: Some("matched allow list".into()),
    };

    assert_roundtrip(&policy);
    assert_roundtrip(&decision);

    // Backward compatibility: legacy payload without status/reasons should still deserialize.
    let legacy = r#"{
        "allow": false,
        "reason": "denied by policy"
    }"#;
    let decoded: PolicyDecision = serde_json::from_str(legacy).expect("legacy decode");
    assert_eq!(decoded.status, PolicyDecisionStatus::Deny);
    assert_eq!(decoded.reason.as_deref(), Some("denied by policy"));
    assert_eq!(decoded.reasons, vec!["denied by policy".to_string()]);
}

#[test]
fn pack_signature_roundtrip() {
    let reference = PackRef::new(
        "oci://registry.greentic.ai/packs/agent",
        Version::parse("1.2.3").expect("semver"),
        "sha256:deadbeef",
    );

    let signature = Signature::new(
        "key-1",
        SignatureAlgorithm::Ed25519,
        vec![0xde, 0xad, 0xbe, 0xef],
    );

    assert_roundtrip(&reference);
    assert_roundtrip(&signature);
}

#[test]
fn span_context_roundtrip() {
    let mut span = SpanContext::new("tenant-2".parse().unwrap(), "flow-alpha", "runtime-core");
    span = span.with_session("sess-9".into()).with_node("node-7");
    #[cfg(feature = "time")]
    {
        let now = OffsetDateTime::from_unix_timestamp(1_700_000_000).expect("timestamp");
        span = span.started(now).finished(now);
    }

    assert_roundtrip(&span);
}

#[test]
fn greentic_error_roundtrip() {
    let err = GreenticError::new(ErrorCode::Internal, "boom");
    let json = serde_json::to_string(&err).expect("serialize");
    let deser: GreenticError = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deser.code, err.code);
    assert_eq!(deser.message, err.message);
}

#[test]
fn tenant_context_summary_roundtrip() {
    let ctx = TenantCtx::new("prod".parse().unwrap(), "tenant-99".parse().unwrap())
        .with_team(Some("team-1".parse().unwrap()))
        .with_user(Some("user-2".parse().unwrap()))
        .with_session("session-3");
    let summary: TenantContext = ctx.tenant_context();
    assert_eq!(summary.tenant_id.as_str(), "tenant-99");
    assert_eq!(
        summary.team_id.as_ref().map(|id| id.as_str()),
        Some("team-1")
    );
    assert_roundtrip(&summary);
}

#[test]
fn semver_req_validates() {
    let req = SemverReq::parse("^1.2").expect("valid semver req");
    assert_eq!(req.to_string(), "^1.2");
    assert!(SemverReq::parse("not-a-semver").is_err());
    let json = serde_json::to_string(&req).expect("serialize");
    assert_eq!(json, "\"^1.2\"");
    assert!(serde_json::from_str::<SemverReq>("\"bad!!\"").is_err());
}

#[test]
fn redaction_path_validates() {
    let path = RedactionPath::parse("$.sensitive.field").expect("valid path");
    assert_eq!(path.as_str(), "$.sensitive.field");
    assert!(RedactionPath::parse("").is_err());
    assert!(RedactionPath::parse("tenant.id").is_err());
    assert_roundtrip(&path);
}

#[test]
fn hash_digest_roundtrip() {
    let digest = HashDigest::blake3("deadbeef").expect("valid hex");
    assert_roundtrip(&digest);
    assert!(HashDigest::blake3("not-hex").is_err());
}

#[test]
fn provider_refs_roundtrip() {
    let git = GitProviderRef::from_str("github").expect("valid git provider ref");
    let scanner = ScannerRef::from_str("trivy").expect("valid scanner ref");

    assert_roundtrip(&git);
    assert_roundtrip(&scanner);

    let git_json = serde_json::to_string(&git).expect("serialize");
    let scanner_json = serde_json::to_string(&scanner).expect("serialize");
    assert_eq!(git_json, "\"github\"");
    assert_eq!(scanner_json, "\"trivy\"");
    assert_eq!(
        serde_json::from_str::<GitProviderRef>(&git_json).expect("deserialize"),
        git
    );
    assert_eq!(
        serde_json::from_str::<ScannerRef>(&scanner_json).expect("deserialize"),
        scanner
    );
}

#[cfg(all(feature = "schemars", feature = "std"))]
#[test]
fn provider_ref_schemas_use_canonical_ids() {
    use greentic_types::{ids, schema};

    let git_schema =
        serde_json::to_value(schema::git_provider_ref()).expect("serialize git schema");
    let scanner_schema =
        serde_json::to_value(schema::scanner_ref()).expect("serialize scanner schema");

    assert_eq!(
        git_schema.get("$id").and_then(|id| id.as_str()),
        Some(ids::GIT_PROVIDER_REF)
    );
    assert_eq!(
        scanner_schema.get("$id").and_then(|id| id.as_str()),
        Some(ids::SCANNER_REF)
    );
}

#[test]
fn pack_id_deserialize_rejects_invalid() {
    let err = serde_json::from_str::<PackId>("\"bad id\"").expect_err("should fail");
    assert!(err.is_data());
}

#[test]
fn semver_req_deserialize_rejects_invalid() {
    let err = serde_json::from_str::<SemverReq>("\"1..0\"").expect_err("should fail");
    assert!(err.is_data());
}

#[test]
fn capabilities_roundtrip() {
    let mut caps = Capabilities::new();
    let mut http = HttpCaps::new();
    http.allow_list = Some(AllowList {
        domains: vec!["api.greentic.ai".into()],
        ports: vec![443],
        protocols: vec![greentic_types::Protocol::Https],
    });
    http.max_body_bytes = Some(1_048_576);
    caps.http = Some(http);

    let mut secrets = SecretsCaps::new();
    secrets.required.push("PRIMARY_TOKEN".into());
    caps.secrets = Some(secrets);

    let mut kv = KvCaps::new();
    kv.namespaces.push("cache".into());
    caps.kv = Some(kv);

    let mut fs = FsCaps::new();
    fs.paths.push("/data".into());
    caps.fs = Some(fs);

    let mut net = NetCaps::new();
    net.policy = Some(NetworkPolicy::strict(AllowList::default()));
    caps.net = Some(net);

    let mut tools = ToolsCaps::new();
    tools.allowed.push("summarize".into());
    caps.tools = Some(tools);

    let mut limits = Limits::new(256, 15_000);
    limits.files = Some(32);
    limits.fuel = Some(10_000);

    let mut telemetry = TelemetrySpec::new("packc");
    telemetry.attributes.insert("env".into(), "dev".into());
    telemetry.emit_node_spans = true;

    assert_roundtrip(&caps);
    assert_roundtrip(&limits);
    assert_roundtrip(&telemetry);
    assert!(!caps.is_empty());
}

#[cfg(feature = "time")]
#[test]
fn run_result_roundtrip() {
    let start = OffsetDateTime::from_unix_timestamp(1_700_000_000).expect("timestamp");
    let finish = start + Duration::seconds(2);
    let summary = NodeSummary {
        node_id: NodeId::from_str("node.entry").unwrap(),
        component: ComponentId::from_str("qa.process").unwrap(),
        status: NodeStatus::Ok,
        duration_ms: 1200,
    };
    let failure = NodeFailure {
        code: "E2E_TEST".into(),
        message: "simulated failure".into(),
        details: BTreeMap::new(),
        transcript_offsets: vec![TranscriptOffset { start: 0, end: 42 }],
        log_paths: vec!["/var/tmp/run.log".into()],
    };

    let result = RunResult {
        session_id: SessionKey::from("sess-42"),
        pack_id: PackId::from_str("greentic.weather.demo").unwrap(),
        pack_version: Version::parse("1.2.3").expect("semver"),
        flow_id: FlowId::from_str("flow-main").unwrap(),
        started_at_utc: start,
        finished_at_utc: finish,
        status: RunStatus::PartialFailure,
        node_summaries: vec![summary],
        failures: vec![failure],
        artifacts_dir: Some("/tmp/run-artifacts".into()),
    };

    assert_roundtrip(&result);
    assert!(result.duration_ms() >= 2000);
}
