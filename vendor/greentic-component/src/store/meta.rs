use anyhow::Result;
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
#[cfg(feature = "abi")]
use std::collections::BTreeSet;
use wasm_metadata::Producers;
#[cfg(feature = "abi")]
use wit_parser::{InterfaceId, PackageName, Resolve, WorldId, WorldItem, WorldKey};

use super::ComponentId;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetaInfo {
    pub id: ComponentId,
    pub size: u64,
    pub abi_version: String,
    pub provider_name: Option<String>,
    pub provider_version: Option<String>,
    pub capabilities: Vec<String>,
}

pub async fn compute_id_and_meta(bytes: &[u8]) -> Result<(ComponentId, MetaInfo)> {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hex::encode(hasher.finalize());
    let id = ComponentId(format!("sha256:{digest}"));

    let size = bytes.len() as u64;

    let mut abi_version = "greentic-abi-0".to_string();
    let mut provider_name = None;
    let mut provider_version = None;
    let mut capabilities = Vec::new();

    #[cfg(feature = "abi")]
    if let Some(extracted) = extract_from_wit_metadata(bytes) {
        abi_version = extracted.abi_version;
        provider_name = extracted.provider_name;
        provider_version = extracted.provider_version;
        capabilities = extracted.capabilities;
    }

    // Fall back to producers metadata to at least capture greentic-interfaces provenance.
    if (provider_name.is_none() || provider_version.is_none())
        && let Ok(Some(producers)) = Producers::from_wasm(bytes)
        && let Some(processed_by) = producers.get("processed-by")
        && let Some(version) = processed_by.get("greentic-interfaces")
    {
        provider_name.get_or_insert_with(|| "greentic-interfaces".to_string());
        provider_version.get_or_insert_with(|| version.clone());
        abi_version = format!("greentic-abi-{}", semver_major(version));
    }

    let meta = MetaInfo {
        id: id.clone(),
        size,
        abi_version,
        provider_name,
        provider_version,
        capabilities,
    };

    Ok((id, meta))
}

struct ExtractedMeta {
    abi_version: String,
    provider_name: Option<String>,
    provider_version: Option<String>,
    capabilities: Vec<String>,
}

#[cfg(feature = "abi")]
fn extract_from_wit_metadata(bytes: &[u8]) -> Option<ExtractedMeta> {
    let decoded = crate::wasm::decode_world(bytes).ok()?;
    let resolve = decoded.resolve;
    let world_id = decoded.world;

    let world = &resolve.worlds[world_id];
    let mut abi_version = "greentic-abi-0".to_string();
    let mut provider_name = None;
    let mut provider_version = None;

    if let Some(pkg_id) = world.package {
        let pkg = &resolve.packages[pkg_id];
        provider_name = Some(pkg.name.name.clone());
        if let Some(version) = &pkg.name.version {
            provider_version = Some(version.to_string());
            abi_version = format!("greentic-abi-{}", version.major);
        }
    }

    let capabilities = collect_import_capabilities(&resolve, world_id);

    Some(ExtractedMeta {
        abi_version,
        provider_name,
        provider_version,
        capabilities,
    })
}

#[cfg(feature = "abi")]
fn collect_import_capabilities(resolve: &Resolve, world_id: WorldId) -> Vec<String> {
    let world = &resolve.worlds[world_id];
    let mut caps = BTreeSet::new();

    for (key, item) in &world.imports {
        match item {
            WorldItem::Interface { id, .. } => {
                caps.insert(interface_label(resolve, *id, key));
            }
            WorldItem::Function(func) => {
                caps.insert(format!("func:{}", func.name));
            }
            WorldItem::Type(_) => {}
        }
    }

    caps.into_iter().collect()
}

#[cfg(feature = "abi")]
fn interface_label(resolve: &Resolve, iface_id: InterfaceId, key: &WorldKey) -> String {
    let iface = &resolve.interfaces[iface_id];
    let name = iface
        .name
        .as_ref()
        .map(|s| s.to_string())
        .or_else(|| key_as_name(key))
        .unwrap_or_else(|| "interface".to_string());

    if let Some(pkg_id) = iface.package {
        let pkg = &resolve.packages[pkg_id];
        format_package(&pkg.name, Some(&name))
    } else {
        name
    }
}

#[cfg(feature = "abi")]
fn format_package(pkg: &PackageName, name: Option<&str>) -> String {
    match (&pkg.name, &pkg.namespace, &pkg.version) {
        (pkg_name, ns, Some(version)) => match name {
            Some(name) => format!("{ns}:{pkg_name}/{name}@{version}"),
            None => format!("{ns}:{pkg_name}@{version}"),
        },
        (pkg_name, ns, None) => match name {
            Some(name) => format!("{ns}:{pkg_name}/{name}"),
            None => format!("{ns}:{pkg_name}"),
        },
    }
}

#[cfg(feature = "abi")]
fn key_as_name(key: &WorldKey) -> Option<String> {
    match key {
        WorldKey::Name(name) => Some(name.to_string()),
        WorldKey::Interface(_) => None,
    }
}

fn semver_major(version: &str) -> u64 {
    Version::parse(version).map(|v| v.major).unwrap_or(0)
}
