#![cfg(feature = "conformance")]

#[test]
fn conformance_feature_placeholder() {
    // Confirm the registry is accessible in conformance builds.
    let registry = greentic_dev::registry::DescribeRegistry::new();
    assert!(registry.get_schema("__nonexistent__").is_none());
}
