#![cfg(feature = "serde")]

use std::collections::BTreeMap;

use greentic_types::{
    ComponentCapabilities, ComponentCapability, ComponentManifest, ComponentOperation,
    ComponentProfiles, DeploymentPlan, Flow, FlowComponentRef, FlowId, FlowKind, FlowMetadata,
    InputMapping, Node, OutputMapping, PackDependency, PackFlowEntry, PackId, PackKind,
    PackManifest, PackSignatures, ResourceHints, Routing, TelemetryHints, decode_pack_manifest,
    encode_pack_manifest,
};
use indexmap::IndexMap;
use semver::Version;
use serde_json::Value;

fn sample_flow() -> Flow {
    let mut nodes: IndexMap<_, _, greentic_types::flow::FlowHasher> = IndexMap::default();
    nodes.insert(
        "start".parse().unwrap(),
        Node {
            id: "start".parse().unwrap(),
            component: FlowComponentRef {
                id: "component.router".parse().unwrap(),
                pack_alias: None,
                operation: Some("route".into()),
            },
            input: InputMapping {
                mapping: serde_json::json!({"input": "value"}),
            },
            output: OutputMapping {
                mapping: Value::Null,
            },
            routing: Routing::Branch {
                on_status: BTreeMap::from([("ok".to_string(), "handler".parse().unwrap())]),
                default: Some("end".parse().unwrap()),
            },
            telemetry: TelemetryHints::default(),
        },
    );
    nodes.insert(
        "handler".parse().unwrap(),
        Node {
            id: "handler".parse().unwrap(),
            component: FlowComponentRef {
                id: "component.handler".parse().unwrap(),
                pack_alias: None,
                operation: None,
            },
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
            component: FlowComponentRef {
                id: "component.end".parse().unwrap(),
                pack_alias: None,
                operation: None,
            },
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

    Flow {
        schema_version: "flow-v1".into(),
        id: "demo.flow".parse().unwrap(),
        kind: FlowKind::Messaging,
        entrypoints: BTreeMap::from([("default".into(), Value::Null)]),
        nodes,
        metadata: FlowMetadata::default(),
    }
}

fn sample_component(id: &str, supports: Vec<FlowKind>) -> ComponentManifest {
    ComponentManifest {
        id: id.parse().unwrap(),
        version: Version::parse("1.0.0").unwrap(),
        supports,
        world: "test:world@1.0.0".into(),
        profiles: ComponentProfiles {
            default: Some("default".into()),
            supported: vec!["default".into()],
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
    }
}

fn sample_pack_manifest() -> PackManifest {
    let flow = sample_flow();
    PackManifest {
        schema_version: "pack-v1".into(),
        pack_id: PackId::new("vendor.demo.pack").unwrap(),
        version: Version::parse("0.1.0").unwrap(),
        kind: PackKind::Application,
        publisher: "vendor".into(),
        components: vec![
            sample_component("component.router", vec![FlowKind::Messaging]),
            sample_component("component.handler", vec![FlowKind::Messaging]),
            sample_component("component.end", vec![FlowKind::Messaging]),
        ],
        flows: vec![PackFlowEntry {
            id: FlowId::new("demo.flow").unwrap(),
            kind: FlowKind::Messaging,
            flow,
            tags: vec!["demo".into()],
            entrypoints: vec!["default".into()],
        }],
        dependencies: vec![PackDependency {
            alias: "provider.messaging".into(),
            pack_id: PackId::new("vendor.messaging").unwrap(),
            version_req: greentic_types::SemverReq::parse("^1.0").unwrap(),
            required_capabilities: vec!["messaging".into()],
        }],
        capabilities: vec![ComponentCapability {
            name: "messaging".into(),
            description: Some("messaging surface".into()),
        }],
        signatures: PackSignatures { signatures: vec![] },
    }
}

fn roundtrip_json<T>(value: &T) -> T
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + core::fmt::Debug,
{
    let json = serde_json::to_string_pretty(value).expect("serialize json");
    let decoded: T = serde_json::from_str(&json).expect("json roundtrip");
    assert_eq!(&decoded, value);
    decoded
}

#[test]
fn pack_manifest_roundtrip_json_and_cbor() {
    let manifest = sample_pack_manifest();
    let json_roundtrip = roundtrip_json(&manifest);
    assert_eq!(json_roundtrip, manifest);

    let bytes = encode_pack_manifest(&manifest).expect("encode");
    let decoded = decode_pack_manifest(&bytes).expect("decode");
    assert_eq!(decoded, manifest);
}

#[test]
fn pack_manifest_cbor_encoding_is_deterministic() {
    let manifest = sample_pack_manifest();
    let first = encode_pack_manifest(&manifest).expect("encode");
    let second = encode_pack_manifest(&manifest).expect("encode");
    assert_eq!(first, second);
}

#[test]
fn deployment_plan_roundtrip_json() {
    let doc = r#"
{
  "pack_id": "vendor.demo",
  "pack_version": "1.2.3",
  "tenant": "tenant-a",
  "environment": "staging",
  "runners": [
    {
      "name": "demo-runner",
      "replicas": 2,
      "capabilities": {
        "can_run_flows": ["flow-a"]
      }
    }
  ],
  "messaging": {
    "logical_cluster": "cluster-1",
    "subjects": [
      {
        "name": "events",
        "purpose": "eventing",
        "durable": true,
        "extra": {}
      }
    ],
    "extra": {}
  },
  "channels": [
    {
      "name": "webchat",
      "flow_id": "demo.flow",
      "kind": "webchat",
      "config": {}
    }
  ],
  "secrets": [
    {
      "key": "API_KEY",
      "required": true,
      "scope": "tenant"
    }
  ],
  "oauth": [
    {
      "provider_id": "generic",
      "logical_client_id": "client-a",
      "redirect_path": "/oauth/callback",
      "extra": {}
    }
  ],
  "telemetry": {
    "required": true,
    "suggested_endpoint": "https://telemetry.local",
    "extra": {}
  },
  "extra": {}
}
"#;

    let plan: DeploymentPlan = serde_json::from_str(doc).expect("valid json");
    let roundtrip = roundtrip_json(&plan);
    assert_eq!(roundtrip.channels.len(), 1);
    assert_eq!(roundtrip.secrets[0].key, "API_KEY");
}
