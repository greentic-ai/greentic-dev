#![cfg(feature = "serde")]

use chrono::{TimeZone, Utc};
use greentic_types::{EventEnvelope, EventId, EventMetadata, TenantCtx};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::json;
use std::fmt::Debug;

fn assert_roundtrip<T>(value: &T)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    let json = serde_json::to_string_pretty(value).expect("serialize");
    let roundtrip: T = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(&roundtrip, value, "{json}");
}

#[test]
fn minimal_event_envelope_roundtrip() {
    let ctx = TenantCtx::new("prod".parse().unwrap(), "tenant-1".parse().unwrap());
    let envelope = EventEnvelope {
        id: EventId::new("evt-1").unwrap(),
        topic: "greentic.repo.build.status".into(),
        r#type: "com.greentic.repo.build.status.v1".into(),
        source: "urn:greentic:repo-service".into(),
        tenant: ctx,
        subject: None,
        time: Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).unwrap(),
        correlation_id: None,
        payload: json!({"status": "ok"}),
        metadata: EventMetadata::new(),
    };

    assert_roundtrip(&envelope);
}

#[test]
fn full_event_envelope_roundtrip_preserves_metadata() {
    let ctx = TenantCtx::new("prod".parse().unwrap(), "tenant-99".parse().unwrap())
        .with_team(Some("team-1".parse().unwrap()))
        .with_user(Some("user-7".parse().unwrap()))
        .with_session("sess-55");
    let mut metadata = EventMetadata::new();
    metadata.insert("idempotency_key".into(), "key-123".into());
    metadata.insert("custom-trace".into(), "trace-abc".into());
    let envelope = EventEnvelope {
        id: EventId::new("evt-42").unwrap(),
        topic: "greentic.repo.build.status".into(),
        r#type: "com.greentic.repo.build.status.v1".into(),
        source: "urn:greentic:repo-service".into(),
        tenant: ctx,
        subject: Some("repo:my-service".into()),
        time: Utc.with_ymd_and_hms(2025, 5, 6, 7, 8, 9).unwrap(),
        correlation_id: Some("corr-9".into()),
        payload: json!({"status": "failed", "attempt": 2}),
        metadata,
    };

    assert_roundtrip(&envelope);
}
