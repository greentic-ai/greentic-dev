#![cfg(feature = "serde")]

use greentic_types::{TenantCtx, WorkerMessage, WorkerRequest, WorkerResponse};
use serde::Serialize;
use serde::de::DeserializeOwned;
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
fn worker_request_roundtrips() {
    let ctx = TenantCtx::new("prod".parse().unwrap(), "tenant-req".parse().unwrap())
        .with_team(Some("team-1".parse().unwrap()))
        .with_user(Some("user-7".parse().unwrap()))
        .with_session("sess-1");
    let request = WorkerRequest {
        version: "1.0".into(),
        tenant: ctx,
        worker_id: "greentic-test-worker".into(),
        correlation_id: Some("corr-123".into()),
        session_id: Some("sess-1".into()),
        thread_id: Some("thread-9".into()),
        payload_json: r#"{"input":"value"}"#.into(),
        timestamp_utc: "2025-01-01T00:00:00Z".into(),
    };

    assert_roundtrip(&request);
}

#[test]
fn worker_response_with_messages_roundtrips() {
    let ctx = TenantCtx::new("prod".parse().unwrap(), "tenant-resp".parse().unwrap());
    let messages = vec![
        WorkerMessage {
            kind: "text".into(),
            payload_json: r#"{"text":"hello"}"#.into(),
        },
        WorkerMessage {
            kind: "card".into(),
            payload_json: r#"{"title":"Card"}"#.into(),
        },
    ];
    let response = WorkerResponse {
        version: "1.0".into(),
        tenant: ctx,
        worker_id: "greentic-test-worker".into(),
        correlation_id: Some("corr-abc".into()),
        session_id: None,
        thread_id: Some("thread-1".into()),
        messages,
        timestamp_utc: "2025-01-01T00:01:00Z".into(),
    };

    assert_roundtrip(&response);
}
