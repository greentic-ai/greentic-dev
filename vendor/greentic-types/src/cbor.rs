//! Canonical CBOR encoding helpers for pack manifests.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;
use core::convert::TryFrom;

use ciborium::{de::from_reader, ser::into_writer};
use indexmap::IndexMap;
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::component::{ComponentOperation, ResourceHints};
use crate::flow::{
    ComponentRef, Flow, FlowHasher, FlowKind, FlowMetadata, InputMapping, Node, OutputMapping,
    Routing, TelemetryHints,
};
use crate::pack_manifest::{
    ComponentCapability, PackDependency, PackFlowEntry, PackManifest, PackSignatures,
};
use crate::{
    ComponentCapabilities, ComponentConfigurators, ComponentId, ComponentManifest,
    ComponentProfiles, FlowId, GreenticError, NodeId, PackId, SemverReq,
};

/// Errors produced while encoding or decoding CBOR manifests.
#[derive(Debug, thiserror::Error)]
pub enum CborError {
    /// CBOR serialization failed.
    #[error("CBOR encode failed: {0}")]
    Encode(String),
    /// CBOR deserialization failed.
    #[error("CBOR decode failed: {0}")]
    Decode(String),
    /// A symbol index referenced an out-of-range entry.
    #[error("invalid symbol index {index} in {table}")]
    InvalidIndex {
        /// Symbol table name.
        table: &'static str,
        /// Offending index.
        index: usize,
    },
    /// Identifier parsing failed during reconstruction.
    #[error("invalid identifier: {0}")]
    InvalidIdentifier(String),
}

/// Canonical encoding entry point.
pub fn encode_pack_manifest(manifest: &PackManifest) -> Result<Vec<u8>, CborError> {
    let encoded = EncodedPackManifest::try_from(manifest)?;
    let mut buf = Vec::new();
    into_writer(&encoded, &mut buf).map_err(|err| CborError::Encode(err.to_string()))?;
    Ok(buf)
}

/// Canonical decoding entry point.
pub fn decode_pack_manifest(bytes: &[u8]) -> Result<PackManifest, CborError> {
    let encoded: EncodedPackManifest =
        from_reader(bytes).map_err(|err| CborError::Decode(err.to_string()))?;
    PackManifest::try_from(encoded)
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct SymbolTables {
    component_ids: Vec<String>,
    node_ids: Vec<String>,
    capability_names: Vec<String>,
    pack_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct EncodedPackManifest {
    schema_version: String,
    pack_id: u32,
    version: String,
    kind: crate::pack_manifest::PackKind,
    publisher: String,
    symbols: SymbolTables,
    components: Vec<EncodedComponent>,
    flows: Vec<EncodedFlowEntry>,
    dependencies: Vec<EncodedDependency>,
    capabilities: Vec<EncodedCapability>,
    signatures: PackSignatures,
}

#[derive(Debug, Serialize, Deserialize)]
struct EncodedComponent {
    id: u32,
    version: String,
    supports: Vec<FlowKind>,
    world: String,
    profiles: ComponentProfiles,
    capabilities: ComponentCapabilities,
    configurators: Option<ComponentConfigurators>,
    operations: Vec<ComponentOperation>,
    config_schema: Option<serde_json::Value>,
    resources: ResourceHints,
}

#[derive(Debug, Serialize, Deserialize)]
struct EncodedFlowEntry {
    id: String,
    kind: FlowKind,
    flow: EncodedFlow,
    tags: Vec<String>,
    entrypoints: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct EncodedFlow {
    schema_version: String,
    id: String,
    kind: FlowKind,
    entrypoints: BTreeMap<String, serde_json::Value>,
    nodes: Vec<EncodedNode>,
    metadata: FlowMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
struct EncodedNode {
    id: u32,
    component: EncodedComponentRef,
    input: InputMapping,
    output: OutputMapping,
    routing: EncodedRouting,
    telemetry: TelemetryHints,
}

#[derive(Debug, Serialize, Deserialize)]
struct EncodedComponentRef {
    id: u32,
    pack_alias: Option<String>,
    operation: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
enum EncodedRouting {
    Next {
        node_id: u32,
    },
    Branch {
        on_status: BTreeMap<String, u32>,
        default: Option<u32>,
    },
    End,
    Reply,
    Custom(serde_json::Value),
}

#[derive(Debug, Serialize, Deserialize)]
struct EncodedDependency {
    alias: String,
    pack_id: u32,
    version_req: String,
    required_capabilities: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct EncodedCapability {
    name: u32,
    description: Option<String>,
}

struct SymbolIndexes {
    component_ids: BTreeMap<String, u32>,
    node_ids: BTreeMap<String, u32>,
    capability_names: BTreeMap<String, u32>,
    pack_ids: BTreeMap<String, u32>,
}

impl TryFrom<&PackManifest> for EncodedPackManifest {
    type Error = CborError;

    fn try_from(manifest: &PackManifest) -> Result<Self, Self::Error> {
        let (symbols, indexes) = build_symbol_tables(manifest);
        let pack_id =
            *indexes
                .pack_ids
                .get(manifest.pack_id.as_str())
                .ok_or(CborError::InvalidIndex {
                    table: "pack_ids",
                    index: usize::MAX,
                })?;

        let components = manifest
            .components
            .iter()
            .map(|component| {
                let id = *indexes.component_ids.get(component.id.as_str()).ok_or(
                    CborError::InvalidIndex {
                        table: "component_ids",
                        index: usize::MAX,
                    },
                )?;
                Ok(EncodedComponent {
                    id,
                    version: component.version.to_string(),
                    supports: component.supports.clone(),
                    world: component.world.clone(),
                    profiles: component.profiles.clone(),
                    capabilities: component.capabilities.clone(),
                    configurators: component.configurators.clone(),
                    operations: component.operations.clone(),
                    config_schema: component.config_schema.clone(),
                    resources: component.resources.clone(),
                })
            })
            .collect::<Result<Vec<_>, CborError>>()?;

        let flows = manifest
            .flows
            .iter()
            .map(|flow_entry| {
                Ok(EncodedFlowEntry {
                    id: flow_entry.id.as_str().to_owned(),
                    kind: flow_entry.kind,
                    flow: encode_flow(&flow_entry.flow, &indexes)?,
                    tags: flow_entry.tags.clone(),
                    entrypoints: flow_entry.entrypoints.clone(),
                })
            })
            .collect::<Result<Vec<_>, CborError>>()?;

        let dependencies =
            manifest
                .dependencies
                .iter()
                .map(|dep| {
                    let pack_id = *indexes.pack_ids.get(dep.pack_id.as_str()).ok_or(
                        CborError::InvalidIndex {
                            table: "pack_ids",
                            index: usize::MAX,
                        },
                    )?;
                    let required_capabilities = dep
                        .required_capabilities
                        .iter()
                        .map(|name| {
                            indexes.capability_names.get(name).copied().ok_or(
                                CborError::InvalidIndex {
                                    table: "capability_names",
                                    index: usize::MAX,
                                },
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(EncodedDependency {
                        alias: dep.alias.clone(),
                        pack_id,
                        version_req: dep.version_req.to_string(),
                        required_capabilities,
                    })
                })
                .collect::<Result<Vec<_>, CborError>>()?;

        let capabilities =
            manifest
                .capabilities
                .iter()
                .map(|cap| {
                    let name = *indexes.capability_names.get(&cap.name).ok_or(
                        CborError::InvalidIndex {
                            table: "capability_names",
                            index: usize::MAX,
                        },
                    )?;
                    Ok(EncodedCapability {
                        name,
                        description: cap.description.clone(),
                    })
                })
                .collect::<Result<Vec<_>, CborError>>()?;

        Ok(EncodedPackManifest {
            schema_version: manifest.schema_version.clone(),
            pack_id,
            version: manifest.version.to_string(),
            kind: manifest.kind,
            publisher: manifest.publisher.clone(),
            symbols,
            components,
            flows,
            dependencies,
            capabilities,
            signatures: manifest.signatures.clone(),
        })
    }
}

fn encode_flow(flow: &Flow, indexes: &SymbolIndexes) -> Result<EncodedFlow, CborError> {
    let nodes = flow
        .nodes
        .iter()
        .map(|(node_id, node)| {
            let id = *indexes
                .node_ids
                .get(node_id.as_str())
                .ok_or(CborError::InvalidIndex {
                    table: "node_ids",
                    index: usize::MAX,
                })?;
            let component_id = *indexes
                .component_ids
                .get(node.component.id.as_str())
                .ok_or(CborError::InvalidIndex {
                    table: "component_ids",
                    index: usize::MAX,
                })?;
            Ok(EncodedNode {
                id,
                component: EncodedComponentRef {
                    id: component_id,
                    pack_alias: node.component.pack_alias.clone(),
                    operation: node.component.operation.clone(),
                },
                input: node.input.clone(),
                output: node.output.clone(),
                routing: encode_routing(&node.routing, indexes)?,
                telemetry: node.telemetry.clone(),
            })
        })
        .collect::<Result<_, CborError>>()?;

    Ok(EncodedFlow {
        schema_version: flow.schema_version.clone(),
        id: flow.id.as_str().to_owned(),
        kind: flow.kind,
        entrypoints: flow.entrypoints.clone(),
        nodes,
        metadata: flow.metadata.clone(),
    })
}

fn encode_routing(routing: &Routing, indexes: &SymbolIndexes) -> Result<EncodedRouting, CborError> {
    match routing {
        Routing::Next { node_id } => {
            let node_id =
                *indexes
                    .node_ids
                    .get(node_id.as_str())
                    .ok_or(CborError::InvalidIndex {
                        table: "node_ids",
                        index: usize::MAX,
                    })?;
            Ok(EncodedRouting::Next { node_id })
        }
        Routing::Branch { on_status, default } => {
            let mut mapped = BTreeMap::new();
            for (status, target) in on_status {
                let idx =
                    *indexes
                        .node_ids
                        .get(target.as_str())
                        .ok_or(CborError::InvalidIndex {
                            table: "node_ids",
                            index: usize::MAX,
                        })?;
                mapped.insert(status.clone(), idx);
            }
            let default = match default {
                Some(node) => Some(*indexes.node_ids.get(node.as_str()).ok_or(
                    CborError::InvalidIndex {
                        table: "node_ids",
                        index: usize::MAX,
                    },
                )?),
                None => None,
            };
            Ok(EncodedRouting::Branch {
                on_status: mapped,
                default,
            })
        }
        Routing::End => Ok(EncodedRouting::End),
        Routing::Reply => Ok(EncodedRouting::Reply),
        Routing::Custom(value) => Ok(EncodedRouting::Custom(value.clone())),
    }
}

impl TryFrom<EncodedPackManifest> for PackManifest {
    type Error = CborError;

    fn try_from(encoded: EncodedPackManifest) -> Result<Self, Self::Error> {
        let pack_id_str = encoded
            .symbols
            .pack_ids
            .get(encoded.pack_id as usize)
            .ok_or(CborError::InvalidIndex {
                table: "pack_ids",
                index: encoded.pack_id as usize,
            })?;
        let pack_id = pack_id_str
            .parse::<PackId>()
            .map_err(|err: GreenticError| CborError::InvalidIdentifier(err.to_string()))?;

        let component_ids = encoded
            .symbols
            .component_ids
            .iter()
            .map(|id| {
                id.parse::<ComponentId>()
                    .map_err(|err: GreenticError| CborError::InvalidIdentifier(err.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let node_ids = encoded
            .symbols
            .node_ids
            .iter()
            .map(|id| {
                id.parse::<NodeId>()
                    .map_err(|err: GreenticError| CborError::InvalidIdentifier(err.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let capability_names = encoded.symbols.capability_names;
        let pack_ids = encoded.symbols.pack_ids;

        let components = encoded
            .components
            .into_iter()
            .map(|component| {
                let id = component_ids
                    .get(component.id as usize)
                    .ok_or(CborError::InvalidIndex {
                        table: "component_ids",
                        index: component.id as usize,
                    })?
                    .clone();
                let version: Version = component
                    .version
                    .parse::<Version>()
                    .map_err(|err| CborError::InvalidIdentifier(err.to_string()))?;
                Ok(ComponentManifest {
                    id,
                    version,
                    supports: component.supports,
                    world: component.world,
                    profiles: component.profiles,
                    capabilities: component.capabilities,
                    configurators: component.configurators,
                    operations: component.operations,
                    config_schema: component.config_schema,
                    resources: component.resources,
                    dev_flows: Default::default(),
                })
            })
            .collect::<Result<Vec<_>, CborError>>()?;

        let flows = encoded
            .flows
            .into_iter()
            .map(|flow_entry| {
                let flow_id = flow_entry
                    .id
                    .parse::<FlowId>()
                    .map_err(|err: GreenticError| CborError::InvalidIdentifier(err.to_string()))?;
                Ok(PackFlowEntry {
                    id: flow_id,
                    kind: flow_entry.kind,
                    flow: decode_flow(flow_entry.flow, &component_ids, &node_ids)?,
                    tags: flow_entry.tags,
                    entrypoints: flow_entry.entrypoints,
                })
            })
            .collect::<Result<Vec<_>, CborError>>()?;

        let dependencies = encoded
            .dependencies
            .into_iter()
            .map(|dep| {
                let pack_id = pack_ids
                    .get(dep.pack_id as usize)
                    .ok_or(CborError::InvalidIndex {
                        table: "pack_ids",
                        index: dep.pack_id as usize,
                    })?
                    .parse::<PackId>()
                    .map_err(|err: GreenticError| CborError::InvalidIdentifier(err.to_string()))?;
                let version_req = SemverReq::parse(dep.version_req)
                    .map_err(|err| CborError::InvalidIdentifier(err.to_string()))?;
                let required_capabilities =
                    dep.required_capabilities
                        .into_iter()
                        .map(|idx| {
                            capability_names.get(idx as usize).cloned().ok_or(
                                CborError::InvalidIndex {
                                    table: "capability_names",
                                    index: idx as usize,
                                },
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                Ok(PackDependency {
                    alias: dep.alias,
                    pack_id,
                    version_req,
                    required_capabilities,
                })
            })
            .collect::<Result<Vec<_>, CborError>>()?;

        let capabilities = encoded
            .capabilities
            .into_iter()
            .map(|cap| {
                let name = capability_names.get(cap.name as usize).cloned().ok_or(
                    CborError::InvalidIndex {
                        table: "capability_names",
                        index: cap.name as usize,
                    },
                )?;
                Ok(ComponentCapability {
                    name,
                    description: cap.description,
                })
            })
            .collect::<Result<Vec<_>, CborError>>()?;

        let version: Version = encoded
            .version
            .parse::<Version>()
            .map_err(|err| CborError::InvalidIdentifier(err.to_string()))?;

        Ok(PackManifest {
            schema_version: encoded.schema_version,
            pack_id,
            version,
            kind: encoded.kind,
            publisher: encoded.publisher,
            components,
            flows,
            dependencies,
            capabilities,
            signatures: encoded.signatures,
        })
    }
}

fn decode_flow(
    flow: EncodedFlow,
    component_ids: &[ComponentId],
    node_ids: &[NodeId],
) -> Result<Flow, CborError> {
    let mut nodes: IndexMap<NodeId, Node, FlowHasher> = IndexMap::default();
    for encoded in flow.nodes {
        let node_id =
            node_ids
                .get(encoded.id as usize)
                .cloned()
                .ok_or(CborError::InvalidIndex {
                    table: "node_ids",
                    index: encoded.id as usize,
                })?;
        let component_id = component_ids
            .get(encoded.component.id as usize)
            .cloned()
            .ok_or(CborError::InvalidIndex {
                table: "component_ids",
                index: encoded.component.id as usize,
            })?;
        let routing = decode_routing(encoded.routing, node_ids)?;
        let node = Node {
            id: node_id.clone(),
            component: ComponentRef {
                id: component_id,
                pack_alias: encoded.component.pack_alias,
                operation: encoded.component.operation,
            },
            input: encoded.input,
            output: encoded.output,
            routing,
            telemetry: encoded.telemetry,
        };
        nodes.insert(node_id, node);
    }

    let id = flow
        .id
        .parse::<FlowId>()
        .map_err(|err: GreenticError| CborError::InvalidIdentifier(err.to_string()))?;

    Ok(Flow {
        schema_version: flow.schema_version,
        id,
        kind: flow.kind,
        entrypoints: flow.entrypoints,
        nodes,
        metadata: flow.metadata,
    })
}

fn decode_routing(encoded: EncodedRouting, node_ids: &[NodeId]) -> Result<Routing, CborError> {
    match encoded {
        EncodedRouting::Next { node_id } => {
            let target =
                node_ids
                    .get(node_id as usize)
                    .cloned()
                    .ok_or(CborError::InvalidIndex {
                        table: "node_ids",
                        index: node_id as usize,
                    })?;
            Ok(Routing::Next { node_id: target })
        }
        EncodedRouting::Branch { on_status, default } => {
            let mut mapped = BTreeMap::new();
            for (status, idx) in on_status {
                let target =
                    node_ids
                        .get(idx as usize)
                        .cloned()
                        .ok_or(CborError::InvalidIndex {
                            table: "node_ids",
                            index: idx as usize,
                        })?;
                mapped.insert(status, target);
            }
            let default = match default {
                Some(idx) => {
                    let node =
                        node_ids
                            .get(idx as usize)
                            .cloned()
                            .ok_or(CborError::InvalidIndex {
                                table: "node_ids",
                                index: idx as usize,
                            })?;
                    Some(node)
                }
                None => None,
            };
            Ok(Routing::Branch {
                on_status: mapped,
                default,
            })
        }
        EncodedRouting::End => Ok(Routing::End),
        EncodedRouting::Reply => Ok(Routing::Reply),
        EncodedRouting::Custom(value) => Ok(Routing::Custom(value)),
    }
}

fn build_symbol_tables(manifest: &PackManifest) -> (SymbolTables, SymbolIndexes) {
    let mut component_ids = BTreeSet::new();
    let mut node_ids = BTreeSet::new();
    let mut capability_names = BTreeSet::new();
    let mut pack_ids = BTreeSet::new();

    pack_ids.insert(manifest.pack_id.as_str().to_owned());

    for component in &manifest.components {
        component_ids.insert(component.id.as_str().to_owned());
    }

    for flow_entry in &manifest.flows {
        for (node_id, node) in &flow_entry.flow.nodes {
            node_ids.insert(node_id.as_str().to_owned());
            component_ids.insert(node.component.id.as_str().to_owned());
        }
    }

    for dep in &manifest.dependencies {
        pack_ids.insert(dep.pack_id.as_str().to_owned());
        for cap in &dep.required_capabilities {
            capability_names.insert(cap.clone());
        }
    }

    for cap in &manifest.capabilities {
        capability_names.insert(cap.name.clone());
    }

    let (component_ids_vec, component_ids_map) = index_from_set(component_ids);
    let (node_ids_vec, node_ids_map) = index_from_set(node_ids);
    let (capability_vec, capability_map) = index_from_set(capability_names);
    let (pack_ids_vec, pack_ids_map) = index_from_set(pack_ids);

    (
        SymbolTables {
            component_ids: component_ids_vec,
            node_ids: node_ids_vec,
            capability_names: capability_vec,
            pack_ids: pack_ids_vec,
        },
        SymbolIndexes {
            component_ids: component_ids_map,
            node_ids: node_ids_map,
            capability_names: capability_map,
            pack_ids: pack_ids_map,
        },
    )
}

fn index_from_set(values: BTreeSet<String>) -> (Vec<String>, BTreeMap<String, u32>) {
    let mut vec = Vec::with_capacity(values.len());
    let mut map = BTreeMap::new();
    for (idx, value) in values.into_iter().enumerate() {
        let idx_u32 = idx as u32;
        map.insert(value.clone(), idx_u32);
        vec.push(value);
    }
    (vec, map)
}
