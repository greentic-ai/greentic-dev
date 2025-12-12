use std::fs;
use std::path::Path;

use greentic_component::schema::{
    collect_capability_hints, collect_default_annotations, collect_redactions,
};

#[test]
fn collects_redactions_and_defaults() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/schemas/redaction.schema.json");
    let raw = fs::read_to_string(path).unwrap();
    let redactions = collect_redactions(&raw);
    let redacted_paths: Vec<_> = redactions.iter().map(|p| p.as_str().to_string()).collect();
    assert_eq!(redacted_paths, vec!["$.credentials.api_key".to_string()]);

    let defaults = collect_default_annotations(&raw).unwrap();
    assert_eq!(defaults[0].0.as_str(), "$.credentials.scope");
    assert_eq!(defaults[0].1, "tenant_scope");
}

#[test]
fn collects_capability_hints() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/schemas/redaction.schema.json");
    let raw = fs::read_to_string(path).unwrap();
    let caps = collect_capability_hints(&raw).unwrap();
    let hints: Vec<_> = caps
        .into_iter()
        .map(|(path, cap)| format!("{}={cap}", path.as_str()))
        .collect();
    assert!(hints.contains(&"$.credentials=secrets".to_string()));
    assert!(hints.contains(&"$.network.hosts[*]=net".to_string()));
}
