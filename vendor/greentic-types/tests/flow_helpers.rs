use std::collections::BTreeMap;

use greentic_types::{
    ComponentCapabilities, ComponentManifest, ComponentOperation, ComponentProfiles, Flow,
    FlowComponentRef, FlowId, FlowKind, FlowMetadata, InputMapping, Node, OutputMapping,
    ResourceHints, Routing, TelemetryHints,
};
use indexmap::IndexMap;
use semver::Version;
use serde_json::{Value, json};

#[test]
fn flow_ingress_respects_insertion_order() {
    let node_a_id: FlowId = "flow.demo".parse().unwrap();
    let mut nodes: IndexMap<_, _, greentic_types::flow::FlowHasher> = IndexMap::default();
    nodes.insert(
        "first".parse().unwrap(),
        Node {
            id: "first".parse().unwrap(),
            component: component_ref("component.first"),
            input: InputMapping {
                mapping: Value::Null,
            },
            output: OutputMapping {
                mapping: Value::Null,
            },
            routing: Routing::Next {
                node_id: "second".parse().unwrap(),
            },
            telemetry: TelemetryHints::default(),
        },
    );
    nodes.insert(
        "second".parse().unwrap(),
        Node {
            id: "second".parse().unwrap(),
            component: component_ref("component.second"),
            input: InputMapping {
                mapping: Value::Null,
            },
            output: OutputMapping {
                mapping: Value::Null,
            },
            routing: Routing::End,
            telemetry: TelemetryHints::default(),
        },
    );

    let flow = Flow {
        schema_version: "flow-v1".into(),
        id: node_a_id,
        kind: FlowKind::Messaging,
        entrypoints: BTreeMap::new(),
        nodes,
        metadata: FlowMetadata::default(),
    };

    let ingress = flow.ingress().expect("ingress");
    assert_eq!(ingress.0.as_str(), "first");
}

#[test]
fn flow_json_roundtrips_with_routing_variants() {
    let mut nodes: IndexMap<_, _, greentic_types::flow::FlowHasher> = IndexMap::default();
    nodes.insert(
        "branch".parse().unwrap(),
        Node {
            id: "branch".parse().unwrap(),
            component: component_ref("component.branch"),
            input: InputMapping {
                mapping: serde_json::json!({"input": "value"}),
            },
            output: OutputMapping {
                mapping: serde_json::json!({"output": "value"}),
            },
            routing: Routing::Branch {
                on_status: BTreeMap::from([("ok".to_string(), "next".parse().unwrap())]),
                default: Some("end".parse().unwrap()),
            },
            telemetry: TelemetryHints::default(),
        },
    );
    nodes.insert(
        "next".parse().unwrap(),
        Node {
            id: "next".parse().unwrap(),
            component: component_ref("component.next"),
            input: InputMapping {
                mapping: Value::Null,
            },
            output: OutputMapping {
                mapping: Value::Null,
            },
            routing: Routing::Reply,
            telemetry: TelemetryHints::default(),
        },
    );
    nodes.insert(
        "end".parse().unwrap(),
        Node {
            id: "end".parse().unwrap(),
            component: component_ref("component.end"),
            input: InputMapping {
                mapping: Value::Null,
            },
            output: OutputMapping {
                mapping: Value::Null,
            },
            routing: Routing::End,
            telemetry: TelemetryHints::default(),
        },
    );

    let flow = Flow {
        schema_version: "flow-v1".into(),
        id: "flow.branching".parse().unwrap(),
        kind: FlowKind::Job,
        entrypoints: BTreeMap::from([("default".into(), Value::Null)]),
        nodes,
        metadata: FlowMetadata::default(),
    };

    let encoded = serde_json::to_string(&flow).expect("serialize");
    let decoded: Flow = serde_json::from_str(&encoded).expect("deserialize");
    assert_eq!(decoded, flow);
}

#[test]
fn component_manifest_defaults_extend() {
    let manifest = ComponentManifest {
        id: "component.profile".parse().unwrap(),
        version: Version::parse("1.0.0").unwrap(),
        supports: vec![FlowKind::Messaging],
        world: "test:component@1.0.0".into(),
        profiles: ComponentProfiles {
            default: Some("default".into()),
            supported: vec!["default".into(), "advanced".into()],
        },
        capabilities: ComponentCapabilities::default(),
        configurators: None,
        operations: vec![ComponentOperation {
            name: "handle".into(),
            input_schema: Value::Null,
            output_schema: Value::Null,
        }],
        config_schema: None,
        resources: ResourceHints::default(),
        dev_flows: Default::default(),
    };

    let default = manifest.select_profile(None).expect("default");
    assert_eq!(default, Some("default"));

    let explicit = manifest.select_profile(Some("advanced")).expect("advanced");
    assert_eq!(explicit, Some("advanced"));

    let err = manifest
        .select_profile(Some("unknown"))
        .expect_err("should fail");
    assert!(matches!(
        err,
        greentic_types::ComponentProfileError::UnsupportedProfile { .. }
    ));
}

#[test]
fn component_manifest_dev_flows_roundtrip() {
    let mut manifest = ComponentManifest {
        id: "component.devflow".parse().unwrap(),
        version: Version::parse("1.2.3").unwrap(),
        supports: vec![FlowKind::Messaging],
        world: "test:component@1.0.0".into(),
        profiles: ComponentProfiles {
            default: Some("default".into()),
            supported: vec!["default".into()],
        },
        capabilities: ComponentCapabilities::default(),
        configurators: None,
        operations: Vec::new(),
        config_schema: None,
        resources: ResourceHints::default(),
        dev_flows: Default::default(),
    };
    manifest.dev_flows.insert(
        "default".parse().unwrap(),
        greentic_types::ComponentDevFlow {
            format: "flow-ir-json".into(),
            graph: json!({
                "nodes": [
                    {"id": "start", "type": "start"},
                    {"id": "end", "type": "end"}
                ],
                "edges": [
                    {"from": "start", "to": "end"}
                ]
            }),
        },
    );

    let encoded = serde_json::to_string(&manifest).expect("serialize");
    let decoded: ComponentManifest = serde_json::from_str(&encoded).expect("deserialize");
    assert_eq!(decoded.dev_flows.len(), 1);
    let default_flow = decoded.dev_flows.get(&"default".parse().unwrap()).unwrap();
    assert_eq!(default_flow.format, "flow-ir-json");
}

fn component_ref(id: &str) -> FlowComponentRef {
    FlowComponentRef {
        id: id.parse().unwrap(),
        pack_alias: None,
        operation: None,
    }
}
