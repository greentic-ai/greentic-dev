#![cfg(feature = "serde")]

use greentic_types::{Attachment, ChannelMessageEnvelope, MessageMetadata, TenantCtx};
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
fn text_only_message_roundtrip() {
    let ctx = TenantCtx::new("prod".parse().unwrap(), "tenant-1".parse().unwrap());
    let envelope = ChannelMessageEnvelope {
        id: "msg-1".into(),
        tenant: ctx,
        channel: "generic-channel".into(),
        session_id: "thread-1".into(),
        user_id: Some("user-1".into()),
        text: Some("hello world".into()),
        attachments: Vec::new(),
        metadata: MessageMetadata::new(),
    };

    assert_roundtrip(&envelope);
}

#[test]
fn message_with_attachments_and_metadata_roundtrip() {
    let ctx = TenantCtx::new("prod".parse().unwrap(), "tenant-9".parse().unwrap())
        .with_team(Some("team-42".parse().unwrap()))
        .with_user(Some("user-22".parse().unwrap()));
    let mut metadata = MessageMetadata::new();
    metadata.insert("correlation_id".into(), "corr-9".into());
    metadata.insert("adapter".into(), "test-adapter".into());
    let attachments = vec![Attachment {
        mime_type: "image/png".into(),
        url: "https://example.test/image.png".into(),
        name: Some("diagram.png".into()),
        size_bytes: Some(1_024),
    }];
    let envelope = ChannelMessageEnvelope {
        id: "msg-attachment".into(),
        tenant: ctx,
        channel: "channel-attachments".into(),
        session_id: "session-44".into(),
        user_id: Some("user-22".into()),
        text: Some("see attachment".into()),
        attachments,
        metadata,
    };

    assert_roundtrip(&envelope);
}
