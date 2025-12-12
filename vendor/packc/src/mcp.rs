use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow, bail};
pub mod adapter_cache;
pub mod adapter_ref;
use adapter_cache::ensure_adapter_local;
use adapter_ref::MCP_ADAPTER_25_06_18;
use semver::Version;
#[cfg(test)]
use serde_json::Map as JsonMap;

use crate::manifest::{McpComponentSpec, SpecBundle, normalize_protocol};
use crate::path_safety::normalize_under_root;

#[derive(Debug, Clone)]
pub struct ComposedMcpComponent {
    pub id: String,
    pub protocol: String,
    pub adapter_template: String,
    pub artifact_path: PathBuf,
    pub version: Version,
}

pub fn compose_all(
    pack_dir: &Path,
    spec_bundle: &SpecBundle,
    pack_version: &Version,
) -> Result<Vec<ComposedMcpComponent>> {
    compose_all_with_override(pack_dir, spec_bundle, pack_version, fake_compose_enabled())
}

pub fn compose_all_with_override(
    pack_dir: &Path,
    spec_bundle: &SpecBundle,
    pack_version: &Version,
    allow_fake_compose: bool,
) -> Result<Vec<ComposedMcpComponent>> {
    if spec_bundle.spec.mcp_components.is_empty() {
        return Ok(Vec::new());
    }

    let workspace = pack_dir.join(".packc").join("mcp");
    fs::create_dir_all(&workspace)
        .with_context(|| format!("failed to prepare MCP workspace {}", workspace.display()))?;

    let mut outputs = Vec::new();

    for entry in &spec_bundle.spec.mcp_components {
        let protocol = normalize_protocol(&entry.protocol);
        let adapter_template = entry.adapter_template.clone();
        let router_path = resolve_router_path(pack_dir, &entry.router_ref)?;
        let adapter_path = resolve_adapter_template(&protocol, &adapter_template)?;
        let out_path = workspace.join(&entry.id).join("component.wasm");

        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create directory {}", parent.display()))?;
        }

        compose_with_wasm_tools(&adapter_path, &router_path, &out_path, allow_fake_compose)
            .with_context(|| format!("failed to compose MCP component `{}`", entry.id))?;

        outputs.push(ComposedMcpComponent {
            id: entry.id.clone(),
            protocol,
            adapter_template,
            artifact_path: out_path,
            version: pack_version.clone(),
        });
    }

    Ok(outputs)
}

fn resolve_router_path(pack_dir: &Path, router_ref: &str) -> Result<PathBuf> {
    let path = normalize_under_root(pack_dir, &PathBuf::from(router_ref))?;
    if path.exists() {
        Ok(path)
    } else {
        bail!(
            "router_ref `{}` does not exist (resolved to {})",
            router_ref,
            path.display()
        );
    }
}

fn resolve_adapter_template(protocol: &str, adapter_template: &str) -> Result<PathBuf> {
    if adapter_template != McpComponentSpec::ADAPTER_DEFAULT {
        bail!(
            "unsupported adapter_template `{}` (only `default` is available)",
            adapter_template
        );
    }

    match protocol {
        McpComponentSpec::PROTOCOL_25_06_18 | McpComponentSpec::PROTOCOL_LATEST => {
            // Today we rely on the vendored adapter; future work may pull from GHCR.
            if let Ok(path) = std::env::var("GREENTIC_PACK_ADAPTER_25_06_18") {
                let adapter_path = PathBuf::from(path);
                if adapter_path.exists() {
                    return Ok(adapter_path);
                }
            }
            ensure_adapter_local(&MCP_ADAPTER_25_06_18)
        }
        other => bail!("unsupported MCP protocol `{}`", other),
    }
}

#[cfg(test)]
fn write_bundled_adapter(workspace: &Path, filename: &str) -> Result<PathBuf> {
    let bundled = include_bytes!("../assets/mcp_adapter_25_06_18.component.wasm");
    let target = workspace.join(filename);
    if !target.exists() {
        fs::write(&target, bundled).with_context(|| {
            format!(
                "failed to materialise bundled adapter at {}",
                target.display()
            )
        })?;
    }
    Ok(target)
}

fn compose_with_wasm_tools(
    adapter: &Path,
    router: &Path,
    output: &Path,
    allow_fake_compose: bool,
) -> Result<()> {
    if allow_fake_compose {
        fs::copy(adapter, output).with_context(|| {
            format!(
                "failed to copy adapter {} to {}",
                adapter.display(),
                output.display()
            )
        })?;
        return Ok(());
    }

    let status = Command::new("wasm-tools")
        .arg("compose")
        .arg(adapter)
        .arg("-d")
        .arg(router)
        .arg("-o")
        .arg(output)
        .status()
        .with_context(|| "failed to invoke `wasm-tools compose`")?;

    if !status.success() {
        bail!("`wasm-tools compose` failed with status {}", status);
    }

    if !output.exists() {
        return Err(anyhow!(
            "expected composed component at {} but it was not produced",
            output.display()
        ));
    }

    Ok(())
}

fn fake_compose_enabled() -> bool {
    std::env::var("PACKC_ALLOW_FAKE_COMPOSE")
        .map(|v| v == "1")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn bundled_adapter_is_written() {
        let temp = tempdir().unwrap();
        let path = write_bundled_adapter(temp.path(), "adapter.wasm").unwrap();
        assert!(path.exists(), "bundled adapter should be written");
        let contents = fs::read(path).unwrap();
        assert!(!contents.is_empty(), "adapter bytes should not be empty");
    }

    #[test]
    fn compose_all_writes_composed_component() {
        let temp = tempdir().unwrap();
        let pack_dir = temp.path();
        let router_path = pack_dir.join("router-component.wasm");
        fs::write(
            &router_path,
            include_bytes!("../assets/mcp_adapter_25_06_18.component.wasm"),
        )
        .unwrap();

        let spec = crate::manifest::PackSpec {
            pack_version: greentic_pack::builder::PACK_VERSION,
            id: "demo.pack".into(),
            version: "0.1.0".into(),
            kind: None,
            name: Some("Demo".into()),
            description: None,
            authors: Vec::new(),
            license: None,
            homepage: None,
            support: None,
            vendor: None,
            flow_files: Vec::new(),
            template_dirs: Vec::new(),
            entry_flows: Vec::new(),
            imports_required: Vec::new(),
            events: None,
            repo: None,
            messaging: None,
            interfaces: Vec::new(),
            mcp_components: vec![McpComponentSpec {
                id: "mcp-demo".into(),
                router_ref: "router-component.wasm".into(),
                protocol: McpComponentSpec::PROTOCOL_25_06_18.into(),
                adapter_template: McpComponentSpec::ADAPTER_DEFAULT.into(),
            }],
            annotations: JsonMap::new(),
            components: Vec::new(),
            distribution: None,
        };

        let specs = crate::manifest::SpecBundle {
            spec,
            source: pack_dir.join("pack.yaml"),
        };

        let composed =
            compose_all_with_override(pack_dir, &specs, &Version::parse("0.1.0").unwrap(), true)
                .expect("composition succeeds");
        assert_eq!(composed.len(), 1, "one mcp component should be produced");
        assert!(
            composed[0].artifact_path.exists(),
            "composed artifact should exist"
        );
    }
}
