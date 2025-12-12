#![cfg(feature = "serde")]

use greentic_types::{
    EventProviderDescriptor, EventProviderKind, OrderingKind, ReliabilityKind, TransportKind,
};
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
fn default_descriptor_roundtrip() {
    let descriptor = EventProviderDescriptor::default();
    assert_roundtrip(&descriptor);
}

#[test]
fn descriptor_with_tags_roundtrip() {
    let descriptor = EventProviderDescriptor {
        name: "nats-core".into(),
        kind: EventProviderKind::Broker,
        transport: TransportKind::Nats,
        reliability: ReliabilityKind::AtLeastOnce,
        ordering: OrderingKind::PerKey,
        notes: Some("core event broker".into()),
        tags: vec!["events".into(), "nats".into()],
    };

    assert_roundtrip(&descriptor);
}
