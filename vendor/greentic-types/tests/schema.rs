#![cfg(feature = "schemars")]

use greentic_types::{Outcome, PackRef, SpanContext, TenantCtx};
use schemars::{JsonSchema, schema_for};
use serde_json::Value;

fn schema_value<T: JsonSchema>() -> Value {
    let schema = schema_for!(T);
    serde_json::to_value(&schema).expect("schema serializes")
}

fn defs_keys(value: &Value) -> Vec<String> {
    value
        .pointer("/$defs")
        .or_else(|| value.pointer("/definitions"))
        .and_then(|defs| defs.as_object())
        .map(|defs| defs.keys().cloned().collect())
        .unwrap_or_default()
}

#[test]
fn tenant_context_schema_registered() {
    let value = schema_value::<TenantCtx>();
    assert!(
        value.is_object(),
        "TenantCtx root schema should be an object"
    );
    let defs = defs_keys(&value);
    assert!(
        defs.iter().any(|name| name.contains("Impersonation")),
        "Impersonation definition missing: {defs:?}"
    );
}

#[test]
fn span_context_schema_has_object() {
    let value = schema_value::<SpanContext>();
    assert!(value.is_object(), "SpanContext schema should be an object");
}

#[test]
fn pack_schema_includes_signature() {
    let value = schema_value::<PackRef>();
    let defs = defs_keys(&value);
    assert!(
        defs.iter().any(|name| name.contains("Signature")),
        "Signature definition missing: {defs:?}"
    );
}

#[test]
fn outcome_schema_enumerates_variants() {
    let value = schema_value::<Outcome<String>>();
    let variants = value
        .pointer("/oneOf")
        .and_then(|variants| variants.as_array())
        .map(|list| list.len())
        .unwrap_or_default();
    assert!(variants >= 3, "Outcome schema should declare variants");
}
