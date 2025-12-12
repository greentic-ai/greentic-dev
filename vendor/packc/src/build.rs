use crate::config::{
    AssetConfig, ComponentConfig, ComponentOperationConfig, FlowConfig, PackConfig,
};
use anyhow::{Context, Result, anyhow};
use greentic_flow::compile_ygtc_str;
use greentic_types::{
    ComponentCapability, ComponentConfigurators, ComponentId, ComponentManifest,
    ComponentOperation, Flow, FlowId, PackDependency, PackFlowEntry, PackId, PackKind,
    PackManifest, PackSignatures, SemverReq, encode_pack_manifest,
};
use semver::Version;
use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tracing::info;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

#[derive(Debug, Clone)]
pub struct BuildOptions {
    pub pack_dir: PathBuf,
    pub component_out: Option<PathBuf>,
    pub manifest_out: PathBuf,
    pub sbom_out: Option<PathBuf>,
    pub gtpack_out: Option<PathBuf>,
    pub dry_run: bool,
}

impl BuildOptions {
    pub fn from_args(args: crate::BuildArgs) -> Result<Self> {
        let pack_dir = args
            .input
            .canonicalize()
            .with_context(|| format!("failed to canonicalize pack dir {}", args.input.display()))?;

        let component_out = args
            .component_out
            .map(|p| if p.is_absolute() { p } else { pack_dir.join(p) });
        let manifest_out = args
            .manifest
            .map(|p| if p.is_relative() { pack_dir.join(p) } else { p })
            .unwrap_or_else(|| pack_dir.join("dist").join("manifest.cbor"));
        let sbom_out = args
            .sbom
            .map(|p| if p.is_absolute() { p } else { pack_dir.join(p) });
        let gtpack_out = args
            .gtpack_out
            .map(|p| if p.is_absolute() { p } else { pack_dir.join(p) });

        Ok(Self {
            pack_dir,
            component_out,
            manifest_out,
            sbom_out,
            gtpack_out,
            dry_run: args.dry_run,
        })
    }
}

pub fn run(opts: &BuildOptions) -> Result<()> {
    info!(
        pack_dir = %opts.pack_dir.display(),
        manifest_out = %opts.manifest_out.display(),
        gtpack_out = ?opts.gtpack_out,
        dry_run = opts.dry_run,
        "building greentic pack"
    );

    let config = crate::config::load_pack_config(&opts.pack_dir)?;
    info!(
        id = %config.pack_id,
        version = %config.version,
        kind = %config.kind,
        components = config.components.len(),
        flows = config.flows.len(),
        dependencies = config.dependencies.len(),
        "loaded pack.yaml"
    );

    let build = assemble_manifest(&config, &opts.pack_dir)?;
    let manifest_bytes = encode_pack_manifest(&build.manifest)?;
    info!(len = manifest_bytes.len(), "encoded manifest.cbor");

    if opts.dry_run {
        info!("dry-run complete; no files written");
        return Ok(());
    }

    if let Some(component_out) = opts.component_out.as_ref() {
        write_stub_wasm(component_out)?;
    }

    write_bytes(&opts.manifest_out, &manifest_bytes)?;

    if let Some(sbom_out) = opts.sbom_out.as_ref() {
        write_bytes(sbom_out, br#"{"files":[]} "#)?;
    }

    if let Some(gtpack_out) = opts.gtpack_out.as_ref() {
        package_gtpack(gtpack_out, &manifest_bytes, &build)?;
        info!(gtpack_out = %gtpack_out.display(), "gtpack archive ready");
    }

    Ok(())
}

struct BuildProducts {
    manifest: PackManifest,
    components: Vec<ComponentBinary>,
    assets: Vec<AssetFile>,
}

struct ComponentBinary {
    id: String,
    source: PathBuf,
}

struct AssetFile {
    logical_path: String,
    source: PathBuf,
}

fn assemble_manifest(config: &PackConfig, pack_root: &Path) -> Result<BuildProducts> {
    let components = build_components(&config.components)?;
    let flows = build_flows(&config.flows)?;
    let dependencies = build_dependencies(&config.dependencies)?;
    let assets = collect_assets(&config.assets, pack_root)?;

    let manifest = PackManifest {
        schema_version: "pack-v1".to_string(),
        pack_id: PackId::new(config.pack_id.clone()).context("invalid pack_id")?,
        version: Version::parse(&config.version)
            .context("invalid pack version (expected semver)")?,
        kind: map_kind(&config.kind)?,
        publisher: config.publisher.clone(),
        components: components.iter().map(|c| c.0.clone()).collect(),
        flows,
        dependencies,
        capabilities: derive_pack_capabilities(&components),
        signatures: PackSignatures::default(),
    };

    Ok(BuildProducts {
        manifest,
        components: components.into_iter().map(|(_, bin)| bin).collect(),
        assets,
    })
}

fn build_components(
    configs: &[ComponentConfig],
) -> Result<Vec<(ComponentManifest, ComponentBinary)>> {
    let mut seen = BTreeSet::new();
    let mut result = Vec::new();

    for cfg in configs {
        if !seen.insert(cfg.id.clone()) {
            anyhow::bail!("duplicate component id {}", cfg.id);
        }

        info!(id = %cfg.id, wasm = %cfg.wasm.display(), "adding component");
        let manifest = ComponentManifest {
            id: ComponentId::new(cfg.id.clone()).context("invalid component id")?,
            version: Version::parse(&cfg.version)
                .context("invalid component version (expected semver)")?,
            supports: cfg.supports.iter().map(|k| k.to_kind()).collect(),
            world: cfg.world.clone(),
            profiles: cfg.profiles.clone(),
            capabilities: cfg.capabilities.clone(),
            configurators: convert_configurators(cfg)?,
            operations: cfg
                .operations
                .iter()
                .map(operation_from_config)
                .collect::<Result<Vec<_>>>()?,
            config_schema: cfg.config_schema.clone(),
            resources: cfg.resources.clone().unwrap_or_default(),
            dev_flows: Default::default(),
        };

        let binary = ComponentBinary {
            id: cfg.id.clone(),
            source: cfg.wasm.clone(),
        };

        result.push((manifest, binary));
    }

    Ok(result)
}

fn operation_from_config(cfg: &ComponentOperationConfig) -> Result<ComponentOperation> {
    Ok(ComponentOperation {
        name: cfg.name.clone(),
        input_schema: cfg.input_schema.clone(),
        output_schema: cfg.output_schema.clone(),
    })
}

fn convert_configurators(cfg: &ComponentConfig) -> Result<Option<ComponentConfigurators>> {
    let Some(configurators) = cfg.configurators.as_ref() else {
        return Ok(None);
    };

    let basic = match &configurators.basic {
        Some(id) => Some(FlowId::new(id).context("invalid configurator flow id")?),
        None => None,
    };
    let full = match &configurators.full {
        Some(id) => Some(FlowId::new(id).context("invalid configurator flow id")?),
        None => None,
    };

    Ok(Some(ComponentConfigurators { basic, full }))
}

fn build_flows(configs: &[FlowConfig]) -> Result<Vec<PackFlowEntry>> {
    let mut seen = BTreeSet::new();
    let mut entries = Vec::new();

    for cfg in configs {
        info!(id = %cfg.id, path = %cfg.file.display(), "compiling flow");
        let yaml_src = fs::read_to_string(&cfg.file)
            .with_context(|| format!("failed to read flow {}", cfg.file.display()))?;

        let flow: Flow = compile_ygtc_str(&yaml_src)
            .with_context(|| format!("failed to compile {}", cfg.file.display()))?;

        let flow_id = flow.id.to_string();
        if !seen.insert(flow_id.clone()) {
            anyhow::bail!("duplicate flow id {}", flow_id);
        }

        let entrypoints = if cfg.entrypoints.is_empty() {
            flow.entrypoints.keys().cloned().collect()
        } else {
            cfg.entrypoints.clone()
        };

        let flow_entry = PackFlowEntry {
            id: flow.id.clone(),
            kind: flow.kind,
            flow,
            tags: cfg.tags.clone(),
            entrypoints,
        };
        entries.push(flow_entry);
    }

    Ok(entries)
}

fn build_dependencies(configs: &[crate::config::DependencyConfig]) -> Result<Vec<PackDependency>> {
    let mut deps = Vec::new();
    let mut seen = BTreeSet::new();
    for cfg in configs {
        if !seen.insert(cfg.alias.clone()) {
            anyhow::bail!("duplicate dependency alias {}", cfg.alias);
        }
        deps.push(PackDependency {
            alias: cfg.alias.clone(),
            pack_id: PackId::new(cfg.pack_id.clone()).context("invalid dependency pack_id")?,
            version_req: SemverReq::parse(&cfg.version_req)
                .context("invalid dependency version requirement")?,
            required_capabilities: cfg.required_capabilities.clone(),
        });
    }
    Ok(deps)
}

fn collect_assets(configs: &[AssetConfig], pack_root: &Path) -> Result<Vec<AssetFile>> {
    let mut assets = Vec::new();
    for cfg in configs {
        let logical = cfg
            .path
            .strip_prefix(pack_root)
            .unwrap_or(&cfg.path)
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("/");
        if logical.is_empty() {
            anyhow::bail!("invalid asset path {}", cfg.path.display());
        }
        assets.push(AssetFile {
            logical_path: logical,
            source: cfg.path.clone(),
        });
    }
    Ok(assets)
}

fn derive_pack_capabilities(
    components: &[(ComponentManifest, ComponentBinary)],
) -> Vec<ComponentCapability> {
    let mut seen = BTreeSet::new();
    let mut caps = Vec::new();

    for (component, _) in components {
        let mut add = |name: &str| {
            if seen.insert(name.to_string()) {
                caps.push(ComponentCapability {
                    name: name.to_string(),
                    description: None,
                });
            }
        };

        if component.capabilities.host.secrets.is_some() {
            add("host:secrets");
        }
        if let Some(state) = &component.capabilities.host.state {
            if state.read {
                add("host:state:read");
            }
            if state.write {
                add("host:state:write");
            }
        }
        if component.capabilities.host.messaging.is_some() {
            add("host:messaging");
        }
        if component.capabilities.host.events.is_some() {
            add("host:events");
        }
        if component.capabilities.host.http.is_some() {
            add("host:http");
        }
        if component.capabilities.host.telemetry.is_some() {
            add("host:telemetry");
        }
        if component.capabilities.host.iac.is_some() {
            add("host:iac");
        }
        if let Some(fs) = component.capabilities.wasi.filesystem.as_ref() {
            add(&format!(
                "wasi:fs:{}",
                format!("{:?}", fs.mode).to_lowercase()
            ));
            if !fs.mounts.is_empty() {
                add("wasi:fs:mounts");
            }
        }
        if component.capabilities.wasi.random {
            add("wasi:random");
        }
        if component.capabilities.wasi.clocks {
            add("wasi:clocks");
        }
    }

    caps
}

fn map_kind(raw: &str) -> Result<PackKind> {
    match raw.to_ascii_lowercase().as_str() {
        "application" => Ok(PackKind::Application),
        "provider" => Ok(PackKind::Provider),
        "infrastructure" => Ok(PackKind::Infrastructure),
        "library" => Ok(PackKind::Library),
        other => Err(anyhow!("unknown pack kind {}", other)),
    }
}

fn package_gtpack(out_path: &Path, manifest_bytes: &[u8], build: &BuildProducts) -> Result<()> {
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let file = fs::File::create(out_path)
        .with_context(|| format!("failed to create {}", out_path.display()))?;
    let mut writer = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .unix_permissions(0o644);

    write_zip_entry(&mut writer, "manifest.cbor", manifest_bytes, options)?;

    let mut component_entries: Vec<_> = build
        .components
        .iter()
        .map(|c| (format!("components/{}.wasm", c.id), c.source.clone()))
        .collect();
    component_entries.sort_by(|a, b| a.0.cmp(&b.0));
    for (logical, source) in component_entries {
        let bytes = fs::read(&source)
            .with_context(|| format!("failed to read component {}", source.display()))?;
        write_zip_entry(&mut writer, &logical, &bytes, options)?;
    }

    let mut asset_entries: Vec<_> = build
        .assets
        .iter()
        .map(|a| (format!("assets/{}", &a.logical_path), a.source.clone()))
        .collect();
    asset_entries.sort_by(|a, b| a.0.cmp(&b.0));
    for (logical, source) in asset_entries {
        let bytes = fs::read(&source)
            .with_context(|| format!("failed to read asset {}", source.display()))?;
        write_zip_entry(&mut writer, &logical, &bytes, options)?;
    }

    writer
        .finish()
        .context("failed to finalise gtpack archive")?;
    Ok(())
}

fn write_zip_entry(
    writer: &mut ZipWriter<std::fs::File>,
    logical_path: &str,
    bytes: &[u8],
    options: SimpleFileOptions,
) -> Result<()> {
    writer
        .start_file(logical_path, options)
        .with_context(|| format!("failed to start {}", logical_path))?;
    writer
        .write_all(bytes)
        .with_context(|| format!("failed to write {}", logical_path))?;
    Ok(())
}

fn write_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory {}", parent.display()))?;
    }
    fs::write(path, bytes).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

fn write_stub_wasm(path: &Path) -> Result<()> {
    const STUB: &[u8] = &[0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
    write_bytes(path, STUB)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn map_kind_accepts_known_values() {
        assert!(matches!(
            map_kind("application").unwrap(),
            PackKind::Application
        ));
        assert!(matches!(map_kind("provider").unwrap(), PackKind::Provider));
        assert!(matches!(
            map_kind("infrastructure").unwrap(),
            PackKind::Infrastructure
        ));
        assert!(matches!(map_kind("library").unwrap(), PackKind::Library));
        assert!(map_kind("unknown").is_err());
    }

    #[test]
    fn collect_assets_preserves_relative_paths() {
        let root = PathBuf::from("/packs/demo");
        let assets = vec![AssetConfig {
            path: root.join("assets").join("foo.txt"),
        }];
        let collected = collect_assets(&assets, &root).expect("collect assets");
        assert_eq!(collected[0].logical_path, "assets/foo.txt");
    }
}
