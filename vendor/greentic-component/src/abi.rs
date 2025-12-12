use std::fs;
use std::path::Path;

use thiserror::Error;
use wit_parser::{Resolve, WorldId, WorldItem};

use crate::lifecycle::Lifecycle;
use crate::wasm::{self, WorldSource};

const WASI_TARGET_MARKER: &str = "wasm32-wasip2";
const DEFAULT_REQUIRED_EXPORTS: [&str; 1] = ["describe"];

#[derive(Debug, Error)]
pub enum AbiError {
    #[error("failed to read component: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },
    #[error("failed to decode embedded component metadata: {0}")]
    Metadata(anyhow::Error),
    #[error("component world mismatch (expected `{expected}`, found `{found}`)")]
    WorldMismatch { expected: String, found: String },
    #[error("invalid world reference `{raw}`; expected namespace:package/world[@version]")]
    InvalidWorldReference { raw: String },
    #[error("component does not export any callable interfaces in `{world}`")]
    MissingExports { world: String },
    #[error("component must target wasm32-wasip2")]
    MissingWasiTarget,
}

pub fn check_world(wasm_path: &Path, expected: &str) -> Result<(), AbiError> {
    let bytes = fs::read(wasm_path)?;
    ensure_wasi_target(&bytes)?;

    let decoded = wasm::decode_world(&bytes).map_err(AbiError::Metadata)?;
    let found = format_world(&decoded.resolve, decoded.world);
    if let WorldSource::Metadata = decoded.source {
        let normalized_expected = normalize_world_ref(expected)?;
        if !worlds_match(&found, &normalized_expected) {
            return Err(AbiError::WorldMismatch {
                expected: normalized_expected,
                found,
            });
        }
    }

    ensure_required_exports(&decoded.resolve, decoded.world, &found)?;
    Ok(())
}

pub fn has_lifecycle(wasm_path: &Path) -> Result<Lifecycle, AbiError> {
    let bytes = fs::read(wasm_path)?;
    let names = extract_export_names(&bytes).unwrap_or_default();
    Ok(Lifecycle {
        init: names.iter().any(|name| name.eq_ignore_ascii_case("init")),
        health: names.iter().any(|name| name.eq_ignore_ascii_case("health")),
        shutdown: names
            .iter()
            .any(|name| name.eq_ignore_ascii_case("shutdown")),
    })
}

fn ensure_wasi_target(bytes: &[u8]) -> Result<(), AbiError> {
    if bytes
        .windows(WASI_TARGET_MARKER.len())
        .any(|window| window == WASI_TARGET_MARKER.as_bytes())
    {
        Ok(())
    } else {
        Err(AbiError::MissingWasiTarget)
    }
}

fn normalize_world_ref(input: &str) -> Result<String, AbiError> {
    let raw = input.trim();
    if !raw.contains('/') {
        return Ok(raw.to_string());
    }
    let (pkg_part, version) = match raw.split_once('@') {
        Some((pkg, ver)) if !pkg.is_empty() && !ver.is_empty() => (pkg, Some(ver)),
        _ => (raw, None),
    };

    let (pkg, world) =
        pkg_part
            .rsplit_once('/')
            .ok_or_else(|| AbiError::InvalidWorldReference {
                raw: input.to_string(),
            })?;
    let (namespace, name) =
        pkg.rsplit_once(':')
            .ok_or_else(|| AbiError::InvalidWorldReference {
                raw: input.to_string(),
            })?;

    let mut id = format!("{namespace}:{name}/{world}");
    if let Some(ver) = version {
        id.push('@');
        id.push_str(ver);
    }
    Ok(id)
}

fn format_world(resolve: &Resolve, world_id: WorldId) -> String {
    let world = &resolve.worlds[world_id];
    if let Some(pkg_id) = world.package {
        let pkg = &resolve.packages[pkg_id];
        if let Some(version) = &pkg.name.version {
            format!(
                "{}:{}/{}@{}",
                pkg.name.namespace, pkg.name.name, world.name, version
            )
        } else {
            format!("{}:{}/{}", pkg.name.namespace, pkg.name.name, world.name)
        }
    } else {
        world.name.clone()
    }
}

fn worlds_match(found: &str, expected: &str) -> bool {
    if found == expected {
        return true;
    }
    let found_base = found.split('@').next().unwrap_or(found);
    let expected_base = expected.split('@').next().unwrap_or(expected);
    if found_base == expected_base {
        return true;
    }
    if !expected_base.contains('/') {
        if let Some((_, world)) = found_base.rsplit_once('/') {
            return world == expected_base;
        }
        return found_base == expected_base;
    }
    false
}

fn ensure_required_exports(
    resolve: &Resolve,
    world_id: WorldId,
    display: &str,
) -> Result<(), AbiError> {
    let world = &resolve.worlds[world_id];
    let has_exports = world.exports.iter().any(|(_, item)| match item {
        WorldItem::Function(_) => true,
        WorldItem::Interface { id, .. } => !resolve.interfaces[*id].functions.is_empty(),
        WorldItem::Type(_) => false,
    });

    if !has_exports {
        return Err(AbiError::MissingExports {
            world: display.to_string(),
        });
    }

    // Soft check for commonly required ops. If the world exports any of
    // these symbols (directly or via interfaces) then we're satisfied.
    let mut satisfied = DEFAULT_REQUIRED_EXPORTS
        .iter()
        .map(|name| (*name, false))
        .collect::<Vec<_>>();

    for (_key, item) in &world.exports {
        match item {
            WorldItem::Function(func) => mark_export(func.name.as_str(), &mut satisfied),
            WorldItem::Interface { id, .. } => {
                for (func, _) in resolve.interfaces[*id].functions.iter() {
                    mark_export(func, &mut satisfied);
                }
            }
            WorldItem::Type(_) => {}
        }

        if satisfied.iter().all(|(_, hit)| *hit) {
            break;
        }
    }

    Ok(())
}

fn mark_export(name: &str, satisfied: &mut [(&str, bool)]) {
    for (needle, flag) in satisfied.iter_mut() {
        if name.eq_ignore_ascii_case(needle) {
            *flag = true;
        }
    }
}

fn extract_export_names(bytes: &[u8]) -> Result<Vec<String>, AbiError> {
    use wasmparser::{ComponentExternalKind, ExternalKind, Parser, Payload};

    let mut names = Vec::new();
    for payload in Parser::new(0).parse_all(bytes) {
        let payload = payload.map_err(|err| AbiError::Metadata(err.into()))?;
        match payload {
            Payload::ComponentExportSection(section) => {
                for export in section {
                    let export = export.map_err(|err| AbiError::Metadata(err.into()))?;
                    if let ComponentExternalKind::Func = export.kind {
                        names.push(export.name.0.to_string());
                    }
                }
            }
            Payload::ExportSection(section) => {
                for export in section {
                    let export = export.map_err(|err| AbiError::Metadata(err.into()))?;
                    if let ExternalKind::Func = export.kind {
                        names.push(export.name.to_string());
                    }
                }
            }
            _ => {}
        }
    }
    Ok(names)
}
