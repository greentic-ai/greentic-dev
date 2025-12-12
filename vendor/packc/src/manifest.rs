use crate::flows::FlowAsset;
use crate::path_safety::normalize_under_root;
use crate::templates::TemplateAsset;
use anyhow::{Context, Result, anyhow};
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use greentic_pack::PackKind;
use greentic_pack::builder::{ComponentDescriptor, DistributionSection, PACK_VERSION};
use greentic_pack::events::EventsSection;
use greentic_pack::messaging::MessagingSection;
use greentic_pack::repo::{InterfaceBinding, RepoPackSection};
use greentic_types::{Signature as SharedSignature, SignatureAlgorithm};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use toml::Value;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PackSpec {
    #[serde(rename = "packVersion")]
    pub pack_version: u32,
    pub id: String,
    pub version: String,
    #[serde(default)]
    #[schemars(with = "Option<String>")]
    pub kind: Option<PackKind>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub support: Option<String>,
    #[serde(default)]
    pub vendor: Option<String>,
    #[serde(default)]
    pub flow_files: Vec<String>,
    #[serde(default)]
    pub template_dirs: Vec<String>,
    #[serde(default)]
    pub entry_flows: Vec<String>,
    #[serde(default)]
    pub imports_required: Vec<String>,
    #[serde(default)]
    pub events: Option<EventsSection>,
    #[serde(default)]
    pub repo: Option<RepoPackSection>,
    #[serde(default)]
    pub messaging: Option<MessagingSection>,
    #[serde(default)]
    pub interfaces: Vec<InterfaceBinding>,
    #[serde(default)]
    pub mcp_components: Vec<McpComponentSpec>,
    #[serde(default)]
    pub annotations: JsonMap<String, JsonValue>,
    #[serde(default)]
    pub components: Vec<ComponentDescriptor>,
    #[serde(default)]
    pub distribution: Option<DistributionSection>,
}

impl PackSpec {
    fn validate(&self) -> Result<()> {
        if self.pack_version != PACK_VERSION {
            anyhow::bail!(
                "unsupported packVersion {}; expected {}",
                self.pack_version,
                PACK_VERSION
            );
        }
        if self.id.trim().is_empty() {
            anyhow::bail!("pack id must not be empty");
        }
        if self.version.trim().is_empty() {
            anyhow::bail!("pack version must not be empty");
        }
        if let Some(kind) = &self.kind {
            kind.validate_allowed()?;
        }
        if let Some(events) = &self.events {
            events.validate()?;
        }
        if let Some(repo) = &self.repo {
            repo.validate()?;
        }
        if let Some(messaging) = &self.messaging {
            messaging.validate()?;
        }
        for binding in &self.interfaces {
            binding.validate("interfaces")?;
        }
        McpComponentSpec::validate_all(&self.mcp_components)?;
        validate_components(&self.components)?;
        validate_distribution(self.kind.as_ref(), self.distribution.as_ref())?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SpecBundle {
    pub spec: PackSpec,
    #[allow(dead_code)]
    pub source: PathBuf,
}

pub fn load_spec(pack_dir: &Path) -> Result<SpecBundle> {
    let manifest_path = normalize_under_root(pack_dir, Path::new("pack.yaml"))?;
    let contents = fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let spec: PackSpec = serde_yaml_bw::from_str(&contents)
        .with_context(|| format!("{} is not a valid PackSpec", manifest_path.display()))?;
    spec.validate()
        .with_context(|| format!("invalid pack spec {}", manifest_path.display()))?;

    Ok(SpecBundle {
        spec,
        source: manifest_path,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackManifest {
    pub pack_id: String,
    pub version: String,
    pub created_at: String,
    pub flows: Vec<FlowEntry>,
    pub templates: Vec<BlobEntry>,
    pub imports_required: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub events: Option<EventsSection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo: Option<RepoPackSection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub messaging: Option<MessagingSection>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mcp_components: Vec<McpComponentManifest>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub components: Vec<ComponentDescriptor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distribution: Option<DistributionSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub flow_type: String,
    pub start: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobEntry {
    pub logical_path: String,
    pub sha256: String,
    pub size: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
pub struct McpComponentSpec {
    pub id: String,
    #[serde(rename = "router_ref")]
    pub router_ref: String,
    #[serde(default = "default_mcp_protocol")]
    pub protocol: String,
    #[serde(default = "default_adapter_template")]
    pub adapter_template: String,
}

impl McpComponentSpec {
    pub const PROTOCOL_25_06_18: &'static str = "25.06.18";
    pub const PROTOCOL_LATEST: &'static str = "latest";
    pub const ADAPTER_DEFAULT: &'static str = "default";

    fn validate_all(specs: &[McpComponentSpec]) -> Result<()> {
        let mut seen_ids = BTreeSet::new();
        for entry in specs {
            if entry.id.trim().is_empty() {
                anyhow::bail!("mcp_components entries must include a non-empty id");
            }
            if !seen_ids.insert(entry.id.clone()) {
                anyhow::bail!("duplicate mcp_components id detected: {}", entry.id);
            }
            if entry.router_ref.trim().is_empty() {
                anyhow::bail!("mcp_components `{}` must specify router_ref", entry.id);
            }
            if entry.router_ref.contains("://") {
                anyhow::bail!(
                    "mcp_components `{}` router_ref must be a local file path; OCI/remote refs are not supported yet: {}",
                    entry.id,
                    entry.router_ref
                );
            }
            let protocol = entry.protocol.trim();
            if protocol != Self::PROTOCOL_25_06_18 && protocol != Self::PROTOCOL_LATEST {
                anyhow::bail!(
                    "mcp_components `{}` uses unsupported protocol `{}` (allowed: 25.06.18, latest)",
                    entry.id,
                    protocol
                );
            }
            let adapter_template = entry.adapter_template.trim();
            if adapter_template != Self::ADAPTER_DEFAULT {
                anyhow::bail!(
                    "mcp_components `{}` uses unsupported adapter_template `{}` (only `default` is available)",
                    entry.id,
                    adapter_template
                );
            }
        }
        Ok(())
    }
}

fn default_mcp_protocol() -> String {
    McpComponentSpec::PROTOCOL_25_06_18.to_string()
}

fn default_adapter_template() -> String {
    McpComponentSpec::ADAPTER_DEFAULT.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpComponentManifest {
    pub id: String,
    pub protocol: String,
    pub adapter_template: String,
}

fn validate_digest(digest: &str) -> Result<()> {
    if digest.trim().is_empty() {
        anyhow::bail!("component digest must not be empty");
    }
    if !digest.starts_with("sha256:") {
        anyhow::bail!("component digest must start with sha256:");
    }
    Ok(())
}

fn validate_component_descriptor(component: &ComponentDescriptor) -> Result<()> {
    if component.component_id.trim().is_empty() {
        anyhow::bail!("component_id must not be empty");
    }
    if component.version.trim().is_empty() {
        anyhow::bail!("component version must not be empty");
    }
    if component.artifact_path.trim().is_empty() {
        anyhow::bail!("component artifact_path must not be empty");
    }
    validate_digest(&component.digest)?;
    if let Some(kind) = &component.kind
        && kind.trim().is_empty()
    {
        anyhow::bail!("component kind must not be empty when provided");
    }
    if let Some(artifact_type) = &component.artifact_type
        && artifact_type.trim().is_empty()
    {
        anyhow::bail!("component artifact_type must not be empty when provided");
    }
    if let Some(platform) = &component.platform
        && platform.trim().is_empty()
    {
        anyhow::bail!("component platform must not be empty when provided");
    }
    if let Some(entrypoint) = &component.entrypoint
        && entrypoint.trim().is_empty()
    {
        anyhow::bail!("component entrypoint must not be empty when provided");
    }
    for tag in &component.tags {
        if tag.trim().is_empty() {
            anyhow::bail!("component tags must not contain empty entries");
        }
    }
    Ok(())
}

fn validate_components(components: &[ComponentDescriptor]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for component in components {
        validate_component_descriptor(component)?;
        let key = (component.component_id.clone(), component.version.clone());
        if !seen.insert(key) {
            anyhow::bail!("duplicate component entry detected");
        }
    }
    Ok(())
}

fn validate_distribution(
    kind: Option<&PackKind>,
    distribution: Option<&DistributionSection>,
) -> Result<()> {
    match (kind, distribution) {
        (Some(PackKind::DistributionBundle), Some(section)) => {
            if let Some(bundle_id) = &section.bundle_id
                && bundle_id.trim().is_empty()
            {
                anyhow::bail!("distribution.bundle_id must not be empty when provided");
            }
            if section.environment_ref.trim().is_empty() {
                anyhow::bail!("distribution.environment_ref must not be empty");
            }
            if section.desired_state_version.trim().is_empty() {
                anyhow::bail!("distribution.desired_state_version must not be empty");
            }
            validate_components(&section.components)?;
            validate_components(&section.platform_components)?;
        }
        (Some(PackKind::DistributionBundle), None) => {
            anyhow::bail!("distribution section is required for kind distribution-bundle");
        }
        (_, Some(_)) => {
            anyhow::bail!("distribution section is only allowed when kind is distribution-bundle");
        }
        _ => {}
    }
    Ok(())
}

pub fn build_manifest(
    bundle: &SpecBundle,
    flows: &[FlowAsset],
    templates: &[TemplateAsset],
) -> PackManifest {
    let created_at = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string());

    let flow_entries = flows
        .iter()
        .map(|flow| FlowEntry {
            id: flow.bundle.id.clone(),
            flow_type: flow.bundle.kind.clone(),
            start: Some(flow.bundle.entry.clone()),
            source: Some(flow.relative_path.to_string_lossy().to_string()),
            sha256: Some(flow.sha256.clone()),
            size: Some(flow.raw.len() as u64),
        })
        .collect();

    let template_entries = templates
        .iter()
        .map(|blob| BlobEntry {
            logical_path: blob.logical_path.clone(),
            sha256: blob.sha256.clone(),
            size: blob.size,
        })
        .collect();

    let mcp_entries = bundle
        .spec
        .mcp_components
        .iter()
        .map(|spec| McpComponentManifest {
            id: spec.id.clone(),
            protocol: normalize_protocol(&spec.protocol),
            adapter_template: spec.adapter_template.clone(),
        })
        .collect();

    PackManifest {
        pack_id: bundle.spec.id.clone(),
        version: bundle.spec.version.clone(),
        created_at,
        flows: flow_entries,
        templates: template_entries,
        imports_required: bundle.spec.imports_required.clone(),
        events: bundle.spec.events.clone(),
        repo: bundle.spec.repo.clone(),
        messaging: bundle.spec.messaging.clone(),
        mcp_components: mcp_entries,
        components: bundle.spec.components.clone(),
        distribution: bundle.spec.distribution.clone(),
    }
}

pub fn encode_manifest(manifest: &PackManifest) -> Result<Vec<u8>> {
    Ok(serde_cbor::to_vec(manifest)?)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackSignature {
    pub alg: String,
    pub key_id: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    pub digest: String,
    pub sig: String,
}

impl PackSignature {
    pub const ED25519: &'static str = "ed25519";

    /// Converts this signature into the shared `greentic-types` representation.
    pub fn to_shared(&self) -> Result<SharedSignature> {
        if self.alg.to_ascii_lowercase() != Self::ED25519 {
            anyhow::bail!("unsupported algorithm {}", self.alg);
        }

        let raw = URL_SAFE_NO_PAD
            .decode(self.sig.as_bytes())
            .map_err(|err| anyhow!("invalid signature encoding: {err}"))?;

        Ok(SharedSignature::new(
            self.key_id.clone(),
            SignatureAlgorithm::Ed25519,
            raw,
        ))
    }
}

pub fn find_manifest_path(pack_dir: &Path) -> Option<PathBuf> {
    MANIFEST_CANDIDATES
        .iter()
        .map(|name| pack_dir.join(name))
        .find(|candidate| candidate.exists())
}

pub fn manifest_path(pack_dir: &Path) -> Result<PathBuf> {
    find_manifest_path(pack_dir)
        .ok_or_else(|| anyhow!("pack manifest not found in {}", pack_dir.display()))
}

pub fn is_pack_manifest_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| {
            MANIFEST_CANDIDATES
                .iter()
                .any(|candidate| candidate == &name)
        })
        .unwrap_or(false)
}

pub fn read_manifest_without_signature(path: &Path) -> Result<Vec<u8>> {
    let mut doc = load_manifest_value(path)?;
    strip_signature(&mut doc);
    let serialized =
        toml::to_string(&doc).map_err(|err| anyhow!("failed to serialise manifest: {err}"))?;
    Ok(serialized.into_bytes())
}

pub fn read_signature(pack_dir: &Path) -> Result<Option<PackSignature>> {
    let Some(path) = find_manifest_path(pack_dir) else {
        return Ok(None);
    };

    let doc = load_manifest_value(&path)?;
    signature_from_doc(&doc)
}

pub fn write_signature(
    pack_dir: &Path,
    signature: &PackSignature,
    out_path: Option<&Path>,
) -> Result<()> {
    let manifest_path = manifest_path(pack_dir)?;
    let mut doc = load_manifest_value(&manifest_path)?;
    set_signature(&mut doc, signature)?;

    let target_path = out_path.unwrap_or(&manifest_path);
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory {}", parent.display()))?;
    }

    let serialized = toml::to_string_pretty(&doc)
        .map_err(|err| anyhow!("failed to serialise manifest: {err}"))?;
    fs::write(target_path, serialized.as_bytes())
        .with_context(|| format!("failed to write {}", target_path.display()))?;

    Ok(())
}

fn load_manifest_value(path: &Path) -> Result<Value> {
    let source =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let table: toml::value::Table =
        toml::from_str(&source).with_context(|| format!("{} is not valid TOML", path.display()))?;
    Ok(Value::Table(table))
}

fn set_signature(doc: &mut Value, signature: &PackSignature) -> Result<()> {
    let table = doc
        .as_table_mut()
        .ok_or_else(|| anyhow!("pack manifest must be a table"))?;

    let greentic_entry = table
        .entry("greentic".to_string())
        .or_insert_with(|| Value::Table(toml::map::Map::new()));

    let greentic_table = greentic_entry
        .as_table_mut()
        .ok_or_else(|| anyhow!("[greentic] must be a table"))?;

    let signature_value = Value::try_from(signature.clone())
        .map_err(|err| anyhow!("failed to serialise signature: {err}"))?;

    greentic_table.insert("signature".to_string(), signature_value);
    Ok(())
}

fn strip_signature(doc: &mut Value) {
    let Some(table) = doc.as_table_mut() else {
        return;
    };

    if let Some(greentic) = table.get_mut("greentic")
        && let Some(section) = greentic.as_table_mut()
    {
        section.remove("signature");
        if section.is_empty() {
            table.remove("greentic");
        }
    }
}

fn signature_from_doc(doc: &Value) -> Result<Option<PackSignature>> {
    let table = doc
        .as_table()
        .ok_or_else(|| anyhow!("pack manifest must be a table"))?;

    let Some(greentic) = table.get("greentic") else {
        return Ok(None);
    };

    let greentic_table = greentic
        .as_table()
        .ok_or_else(|| anyhow!("[greentic] must be a table"))?;

    let Some(signature_value) = greentic_table.get("signature") else {
        return Ok(None);
    };

    let signature: PackSignature = signature_value
        .clone()
        .try_into()
        .map_err(|err| anyhow!("invalid greentic.signature block: {err}"))?;

    Ok(Some(signature))
}

const MANIFEST_CANDIDATES: [&str; 2] = ["pack.toml", "greentic-pack.toml"];

pub fn normalize_protocol(raw: &str) -> String {
    if raw.trim() == McpComponentSpec::PROTOCOL_LATEST {
        McpComponentSpec::PROTOCOL_25_06_18.to_string()
    } else {
        raw.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{flows, templates};

    fn demo_pack_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("examples/weather-demo")
    }

    #[test]
    fn weather_demo_manifest_includes_mcp_exec_flow() {
        let pack_dir = demo_pack_dir();
        let spec_bundle = load_spec(&pack_dir).expect("spec loads");
        assert_eq!(spec_bundle.spec.id, "greentic.weather.demo");

        let flows = flows::load_flows(&pack_dir, &spec_bundle.spec).expect("flows load");
        assert_eq!(flows.len(), 1);
        assert!(
            flows[0].raw.contains("mcp.exec"),
            "flow should reference mcp.exec node"
        );

        let templates =
            templates::collect_templates(&pack_dir, &spec_bundle.spec).expect("templates load");
        assert_eq!(templates.len(), 1);

        let manifest = build_manifest(&spec_bundle, &flows, &templates);
        assert_eq!(manifest.flows[0].id, "weather_bot");
        assert_eq!(manifest.flows[0].flow_type, "messaging");
        assert_eq!(manifest.flows[0].start.as_deref(), Some("collect_location"));
        assert_eq!(
            manifest.flows[0].source.as_deref(),
            Some("flows/weather_bot.ygtc")
        );
        assert_eq!(
            manifest.templates[0].logical_path,
            "templates/weather_now.hbs"
        );
        assert_eq!(manifest.imports_required.len(), 2);

        let encoded = encode_manifest(&manifest).expect("manifest encodes");
        assert!(!encoded.is_empty(), "CBOR output should not be empty");
    }

    #[test]
    fn mcp_components_are_normalized() {
        let spec = PackSpec {
            pack_version: PACK_VERSION,
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
                router_ref: "router.component.wasm".into(),
                protocol: McpComponentSpec::PROTOCOL_LATEST.into(),
                adapter_template: McpComponentSpec::ADAPTER_DEFAULT.into(),
            }],
            annotations: JsonMap::new(),
            components: Vec::new(),
            distribution: None,
        };
        let bundle = SpecBundle {
            spec,
            source: PathBuf::from("pack.yaml"),
        };
        let manifest = build_manifest(&bundle, &[], &[]);
        assert_eq!(manifest.mcp_components.len(), 1);
        assert_eq!(
            manifest.mcp_components[0].protocol,
            McpComponentSpec::PROTOCOL_25_06_18
        );
    }

    #[test]
    fn distribution_bundle_with_distribution_section_validates() {
        let spec = PackSpec {
            pack_version: PACK_VERSION,
            id: "bundle.pack".into(),
            version: "0.1.0".into(),
            kind: Some(PackKind::DistributionBundle),
            name: None,
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
            mcp_components: Vec::new(),
            annotations: JsonMap::new(),
            components: Vec::new(),
            distribution: Some(DistributionSection {
                bundle_id: Some("bundle-123".into()),
                tenant: JsonMap::new(),
                environment_ref: "env-prod".into(),
                desired_state_version: "v1".into(),
                components: vec![ComponentDescriptor {
                    component_id: "comp.a".into(),
                    version: "1.0.0".into(),
                    digest: "sha256:abc".into(),
                    artifact_path: "artifacts/comp.a".into(),
                    kind: Some("software".into()),
                    artifact_type: Some("binary/linux-x86_64".into()),
                    tags: vec!["runner-dependency".into()],
                    platform: None,
                    entrypoint: Some("install.sh".into()),
                }],
                platform_components: Vec::new(),
            }),
        };
        spec.validate().expect("distribution bundle validates");
    }

    #[test]
    fn software_component_descriptor_validates() {
        let components = vec![ComponentDescriptor {
            component_id: "install.demo".into(),
            version: "1.2.3".into(),
            digest: "sha256:def".into(),
            artifact_path: "artifacts/demo.bin".into(),
            kind: Some("software".into()),
            artifact_type: Some("binary/linux-x86_64".into()),
            tags: vec!["platform".into(), "runner-dependency".into()],
            platform: Some("linux-x86_64".into()),
            entrypoint: Some("install.sh".into()),
        }];
        validate_components(&components).expect("components validate");
    }
}
