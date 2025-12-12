use std::fs;
use std::path::Path;

use greentic_component::manifest::{
    DescribeKind, ManifestError, parse_manifest, validate_manifest,
};
use greentic_types::flow::FlowKind;
use serde_json::Value;

fn fixture(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/manifests")
        .join(name);
    fs::read_to_string(path).expect("fixture should exist")
}

#[test]
fn round_trip_manifest_parse() {
    let raw = fixture("valid.component.json");
    let manifest = parse_manifest(&raw).expect("manifest parses");
    assert_eq!(manifest.id.as_str(), "com.greentic.demo.echo");
    assert_eq!(manifest.version.to_string(), "0.3.0");
    assert_eq!(manifest.describe_export.kind(), DescribeKind::Export);
    assert_eq!(
        manifest.supports,
        vec![FlowKind::Messaging, FlowKind::Event]
    );
    assert_eq!(
        manifest.profiles.supported,
        vec!["stateless".to_string(), "cached".to_string()]
    );
    assert!(manifest.telemetry.is_some());
}

#[test]
fn schema_validation_fails_for_missing_fields() {
    let raw = fixture("invalid.component.json");
    match parse_manifest(&raw).unwrap_err() {
        ManifestError::Schema(_) => {}
        err => panic!("expected schema error, got {err:?}"),
    }
}

#[test]
fn semver_validation_reports_leading_zero() {
    let raw = fixture("valid.component.json");
    let mut value: Value = serde_json::from_str(&raw).unwrap();
    value["version"] = Value::String("01.0.0".into());
    let raw_with_bad_version = serde_json::to_string(&value).unwrap();
    match parse_manifest(&raw_with_bad_version).unwrap_err() {
        ManifestError::InvalidVersion { .. } => {}
        err => panic!("expected InvalidVersion, got {err:?}"),
    }
}

#[test]
fn relative_artifact_path_required() {
    let raw = fixture("valid.component.json");
    let mut value: Value = serde_json::from_str(&raw).unwrap();
    value["artifacts"]["component_wasm"] = Value::String("/abs/component.wasm".into());
    let serialized = serde_json::to_string(&value).unwrap();
    match parse_manifest(&serialized).unwrap_err() {
        ManifestError::InvalidArtifactPath { .. } => {}
        err => panic!("expected InvalidArtifactPath, got {err:?}"),
    }
}

#[test]
fn manifest_schema_helper_exposes_json() {
    assert!(greentic_component::manifest_schema().contains("$schema"));
}

#[test]
fn validate_manifest_round_trip() {
    let raw = fixture("valid.component.json");
    validate_manifest(&raw).expect("schema-valid manifest");
}
