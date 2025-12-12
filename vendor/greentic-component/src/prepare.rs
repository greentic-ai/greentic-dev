use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use dashmap::DashMap;
use once_cell::sync::Lazy;

#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::abi;
use crate::capabilities::Capabilities;
use crate::describe::{self, DescribePayload};
use crate::error::ComponentError;
use crate::lifecycle::Lifecycle;
use crate::limits::Limits;
use crate::loader;
use crate::manifest::ComponentManifest;
use crate::schema::{self, JsonPath};
use crate::signing::{SigningError, compute_wasm_hash};
use crate::telemetry::TelemetrySpec;

#[derive(Debug, Clone)]
pub struct PreparedComponent {
    pub manifest: ComponentManifest,
    pub manifest_path: PathBuf,
    pub wasm_path: PathBuf,
    pub root: PathBuf,
    pub wasm_hash: String,
    pub describe: DescribePayload,
    pub lifecycle: Lifecycle,
    pub redactions: Vec<JsonPath>,
    pub defaults: Vec<String>,
    pub hash_verified: bool,
    pub world_ok: bool,
}

static ABI_CACHE: Lazy<DashMap<(PathBuf, String), FileStamp>> = Lazy::new(DashMap::new);
static DESCRIBE_CACHE: Lazy<DashMap<PathBuf, DescribeCacheEntry>> = Lazy::new(DashMap::new);

#[cfg(test)]
static ABI_MISSES: Lazy<AtomicUsize> = Lazy::new(|| AtomicUsize::new(0));
#[cfg(test)]
static DESCRIBE_MISSES: Lazy<AtomicUsize> = Lazy::new(|| AtomicUsize::new(0));

pub fn prepare_component(path_or_id: &str) -> Result<PreparedComponent, ComponentError> {
    let handle = loader::discover(path_or_id)?;
    let manifest = handle.manifest.clone();
    let manifest_path = handle.manifest_path.clone();
    let root = handle.root.clone();
    let wasm_path = handle.wasm_path.clone();

    let computed_hash = compute_wasm_hash(&wasm_path)?;
    if computed_hash != manifest.hashes.component_wasm.as_str() {
        return Err(SigningError::HashMismatch {
            expected: manifest.hashes.component_wasm.as_str().to_string(),
            found: computed_hash,
        }
        .into());
    }

    cached_world_check(&wasm_path, manifest.world.as_str())?;
    let lifecycle = abi::has_lifecycle(&wasm_path)?;
    let describe_payload = cached_describe(&wasm_path, &manifest)?;
    let mut redactions = Vec::new();
    let mut defaults = Vec::new();
    for version in &describe_payload.versions {
        let schema_str = serde_json::to_string(&version.schema)
            .expect("describe schema serialization never fails");
        let mut hits = schema::try_collect_redactions(&schema_str)?;
        redactions.append(&mut hits);
        let defaults_hits = schema::collect_default_annotations(&schema_str)?;
        defaults.extend(
            defaults_hits
                .into_iter()
                .map(|(path, applied)| format!("{}={}", path.as_str(), applied)),
        );
    }

    Ok(PreparedComponent {
        manifest,
        manifest_path,
        wasm_path,
        root,
        wasm_hash: computed_hash,
        describe: describe_payload,
        lifecycle,
        redactions,
        defaults,
        hash_verified: true,
        world_ok: true,
    })
}

fn cached_world_check(path: &Path, expected: &str) -> Result<(), ComponentError> {
    let stamp = file_stamp(path)?;
    let key = (path.to_path_buf(), expected.to_string());
    if let Some(entry) = ABI_CACHE.get(&key)
        && *entry == stamp
    {
        return Ok(());
    }

    abi::check_world(path, expected)?;
    #[cfg(test)]
    {
        ABI_MISSES.fetch_add(1, Ordering::SeqCst);
    }
    ABI_CACHE.insert(key, stamp);
    Ok(())
}

fn cached_describe(
    path: &Path,
    manifest: &ComponentManifest,
) -> Result<DescribePayload, ComponentError> {
    let stamp = file_stamp(path)?;
    if let Some(entry) = DESCRIBE_CACHE.get(path)
        && entry.stamp == stamp
        && entry.export == manifest.describe_export.as_str()
    {
        return Ok(entry.payload.clone());
    }

    let payload = describe::load(path, manifest)?;
    #[cfg(test)]
    {
        DESCRIBE_MISSES.fetch_add(1, Ordering::SeqCst);
    }
    DESCRIBE_CACHE.insert(
        path.to_path_buf(),
        DescribeCacheEntry {
            stamp,
            export: manifest.describe_export.as_str().to_string(),
            payload: payload.clone(),
        },
    );
    Ok(payload)
}

fn file_stamp(path: &Path) -> Result<FileStamp, ComponentError> {
    let meta = fs::metadata(path)?;
    let len = meta.len();
    let modified = meta
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|dur| dur.as_nanos())
        .unwrap_or(0);
    Ok(FileStamp { len, modified })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct FileStamp {
    len: u64,
    modified: u128,
}

#[derive(Clone)]
struct DescribeCacheEntry {
    stamp: FileStamp,
    export: String,
    payload: DescribePayload,
}

pub fn clear_cache_for(path: &Path) {
    let path_buf = path.to_path_buf();
    ABI_CACHE.retain(|(p, _), _| p != &path_buf);
    DESCRIBE_CACHE.remove(path);
}

#[derive(Debug, Clone)]
pub struct RunnerConfig {
    pub wasm_path: PathBuf,
    pub world: String,
    pub capabilities: Capabilities,
    pub limits: Option<Limits>,
    pub telemetry: Option<TelemetrySpec>,
    pub redactions: Vec<JsonPath>,
    pub defaults: Vec<String>,
    pub describe: DescribePayload,
}

#[derive(Debug, Clone)]
pub struct PackEntry {
    pub manifest_json: String,
    pub describe_schema: Option<String>,
    pub wasm_hash: String,
    pub world: String,
}

impl PreparedComponent {
    pub fn redaction_paths(&self) -> &[JsonPath] {
        &self.redactions
    }

    pub fn defaults_applied(&self) -> &[String] {
        &self.defaults
    }

    pub fn to_runner_config(&self) -> RunnerConfig {
        RunnerConfig {
            wasm_path: self.wasm_path.clone(),
            world: self.manifest.world.as_str().to_string(),
            capabilities: self.manifest.capabilities.clone(),
            limits: self.manifest.limits.clone(),
            telemetry: self.manifest.telemetry.clone(),
            redactions: self.redactions.clone(),
            defaults: self.defaults.clone(),
            describe: self.describe.clone(),
        }
    }

    pub fn to_pack_entry(&self) -> Result<PackEntry, ComponentError> {
        let manifest_json = fs::read_to_string(&self.manifest_path)?;
        let describe_schema = self.describe.versions.first().map(|version| {
            serde_json::to_string(&version.schema).expect("describe schema serialization")
        });
        Ok(PackEntry {
            manifest_json,
            describe_schema,
            wasm_hash: self.wasm_hash.clone(),
            world: self.manifest.world.as_str().to_string(),
        })
    }
}

#[cfg(test)]
pub(crate) fn cache_stats() -> (usize, usize) {
    (
        ABI_MISSES.load(Ordering::SeqCst),
        DESCRIBE_MISSES.load(Ordering::SeqCst),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use blake3::Hasher;
    use tempfile::TempDir;
    use wasm_encoder::{
        CodeSection, CustomSection, ExportKind, ExportSection, Function, FunctionSection,
        Instruction, Module, TypeSection,
    };
    use wit_component::{StringEncoding, metadata};
    use wit_parser::{Resolve, WorldId};

    const TEST_WIT: &str = r#"
package greentic:component@0.1.0;
world node {
    export describe: func();
}
"#;

    #[test]
    fn caches_results() {
        ABI_MISSES.store(0, Ordering::SeqCst);
        DESCRIBE_MISSES.store(0, Ordering::SeqCst);
        let fixture = TestFixture::new(TEST_WIT, &["describe"]);
        prepare_component(fixture.manifest_path.to_str().unwrap()).unwrap();
        let first = cache_stats();
        prepare_component(fixture.manifest_path.to_str().unwrap()).unwrap();
        assert_eq!(first, cache_stats());
    }

    struct TestFixture {
        _temp: TempDir,
        manifest_path: PathBuf,
    }

    impl TestFixture {
        fn new(world_src: &str, funcs: &[&str]) -> Self {
            let temp = TempDir::new().expect("tempdir");
            let (wasm, manifest) = build_component(world_src, funcs);
            fs::write(temp.path().join("component.wasm"), &wasm).unwrap();
            let manifest_path = temp.path().join("component.manifest.json");
            fs::write(&manifest_path, manifest).unwrap();
            Self {
                _temp: temp,
                manifest_path,
            }
        }
    }

    fn build_component(world_src: &str, funcs: &[&str]) -> (Vec<u8>, String) {
        let mut resolve = Resolve::default();
        let pkg = resolve.push_str("test.wit", world_src).unwrap();
        let world = resolve.select_world(&[pkg], Some("node")).unwrap();
        let metadata = metadata::encode(&resolve, world, StringEncoding::UTF8, None).unwrap();

        let mut module = Module::new();
        let mut types = TypeSection::new();
        types.ty().function([], []);
        module.section(&types);

        let mut funcs_section = FunctionSection::new();
        for _ in funcs {
            funcs_section.function(0);
        }
        module.section(&funcs_section);

        let mut exports = ExportSection::new();
        for (idx, name) in funcs.iter().enumerate() {
            exports.export(name, ExportKind::Func, idx as u32);
        }
        module.section(&exports);

        let mut code = CodeSection::new();
        for _ in funcs {
            let mut body = Function::new([]);
            body.instruction(&Instruction::End);
            code.function(&body);
        }
        module.section(&code);

        module.section(&CustomSection {
            name: "component-type".into(),
            data: std::borrow::Cow::Borrowed(&metadata),
        });
        module.section(&CustomSection {
            name: "producers".into(),
            data: std::borrow::Cow::Borrowed(b"wasm32-wasip2"),
        });

        let wasm_bytes = module.finish();
        let observed_world = detect_world(&wasm_bytes).unwrap_or_else(|| "root:root/root".into());
        let mut hasher = Hasher::new();
        hasher.update(&wasm_bytes);
        let digest = hasher.finalize();
        let hash = format!("blake3:{}", hex::encode(digest.as_bytes()));

        let manifest = serde_json::json!({
            "id": "com.greentic.test.component",
            "name": "Test",
            "version": "0.1.0",
            "world": observed_world,
            "describe_export": "describe",
            "supports": ["messaging"],
            "profiles": {
                "default": "stateless",
                "supported": ["stateless"]
            },
            "capabilities": {
                "wasi": {
                    "filesystem": {
                        "mode": "none",
                        "mounts": []
                    },
                    "random": true,
                    "clocks": true
                },
                "host": {
                    "messaging": {
                        "inbound": true,
                        "outbound": true
                    }
                }
            },
            "limits": {"memory_mb": 64, "wall_time_ms": 1000},
            "telemetry": {"span_prefix": "test.component"},
            "artifacts": {"component_wasm": "component.wasm"},
            "hashes": {"component_wasm": hash},
        });

        (wasm_bytes, serde_json::to_string_pretty(&manifest).unwrap())
    }

    fn detect_world(bytes: &[u8]) -> Option<String> {
        let decoded = crate::wasm::decode_world(bytes).ok()?;
        Some(world_label(&decoded.resolve, decoded.world))
    }

    fn world_label(resolve: &Resolve, world_id: WorldId) -> String {
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
}
