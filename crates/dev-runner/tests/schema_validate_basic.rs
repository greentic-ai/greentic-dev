#[test]
fn schema_validation_passes_for_examples() {
    let yaml = std::fs::read_to_string("examples/flows/min.yaml").unwrap();
    let doc: serde_yaml_bw::Value = serde_yaml_bw::from_str(&yaml).unwrap();
    let schema = r#"{"type":"object"}"#; // placeholder schema
    assert!(dev_runner::schema::validate_yaml_against_schema(&doc, schema).is_ok());
}
