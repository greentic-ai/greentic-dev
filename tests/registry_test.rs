use greentic_dev::registry::DescribeRegistry;

#[test]
fn can_get_stub_schema() {
    let registry = DescribeRegistry::new();
    assert!(registry.get_schema("oauth").is_some());
}
