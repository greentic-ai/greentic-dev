#![cfg(test)]

use greentic_types::{PackId, SemverReq};
use proptest::prelude::*;
use std::str::FromStr;

fn valid_id_strings() -> impl Strategy<Value = String> {
    prop::collection::vec(valid_id_char(), 1..32).prop_map(|chars| chars.into_iter().collect())
}

fn valid_id_char() -> impl Strategy<Value = char> {
    prop::num::u8::ANY.prop_filter_map("valid id char", |byte| {
        let c = byte as char;
        if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
            Some(c)
        } else {
            None
        }
    })
}

fn invalid_id_strings() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(String::new()),
        prop::collection::vec(
            prop_oneof![
                Just(' '),
                Just('\n'),
                Just('\t'),
                Just('@'),
                Just('!'),
                Just('/'),
                Just('é')
            ],
            1..16
        )
        .prop_map(|chars| chars.into_iter().collect()),
    ]
}

fn version_component() -> impl Strategy<Value = u32> {
    0u32..1000
}

fn version_string() -> impl Strategy<Value = String> {
    (
        version_component(),
        version_component(),
        version_component(),
    )
        .prop_map(|(major, minor, patch)| format!("{major}.{minor}.{patch}"))
}

fn comparator() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just(""),
        Just("^"),
        Just("~"),
        Just(">="),
        Just("<="),
        Just(">"),
        Just("<"),
        Just("="),
    ]
}

fn valid_semver_strings() -> impl Strategy<Value = String> {
    (comparator(), version_string()).prop_map(|(cmp, version)| format!("{cmp}{version}"))
}

fn invalid_semver_strings() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(String::new()),
        Just("invalid requirement".to_string()),
        Just("1..0".to_string()),
        Just("v1.0.0".to_string()),
        Just("> =1.0.0".to_string()),
        prop::collection::vec(
            prop_oneof![Just(' '), Just('\t'), Just('!'), Just('@')],
            2..8
        )
        .prop_map(|chars| chars.into_iter().collect()),
    ]
}

proptest! {
    #[test]
    fn pack_id_roundtrip(raw in valid_id_strings()) {
        let parsed = PackId::from_str(&raw).expect("strategy builds valid IDs");
        prop_assert_eq!(parsed.as_str(), raw.as_str());
        let reparsed: PackId = raw.as_str().parse().unwrap();
        prop_assert_eq!(reparsed.as_str(), parsed.as_str());
    }

    #[test]
    fn pack_id_rejects_invalid(raw in invalid_id_strings()) {
        prop_assert!(PackId::from_str(&raw).is_err());
    }

    #[test]
    fn semver_req_roundtrip(raw in valid_semver_strings()) {
        let parsed = SemverReq::parse(&raw).expect("valid semver req");
        let canonical = parsed.to_string();
        prop_assert_eq!(canonical.as_str(), raw.as_str());
        let via_from_str: SemverReq = raw.as_str().parse().unwrap();
        prop_assert_eq!(via_from_str, parsed);
    }

    #[test]
    fn semver_req_rejects_invalid(raw in invalid_semver_strings()) {
        prop_assert!(SemverReq::parse(&raw).is_err());
    }
}
