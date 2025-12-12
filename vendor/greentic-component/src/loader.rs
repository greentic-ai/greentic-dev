use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use directories::BaseDirs;
use thiserror::Error;

use crate::manifest::{ComponentManifest, parse_manifest};
use crate::signing::{SigningError, verify_manifest_hash};

const MANIFEST_NAME: &str = "component.manifest.json";

#[derive(Debug, Clone)]
pub struct ComponentHandle {
    pub manifest: ComponentManifest,
    pub wasm_path: PathBuf,
    pub root: PathBuf,
    pub manifest_path: PathBuf,
}

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("component not found for `{0}`")]
    NotFound(String),
    #[error("failed to read {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("manifest parse failed at {path}: {source}")]
    Manifest {
        path: PathBuf,
        #[source]
        source: crate::manifest::ManifestError,
    },
    #[error("missing artifact `{path}` declared in manifest")]
    MissingArtifact { path: PathBuf },
    #[error("hash verification failed: {0}")]
    Signing(#[from] SigningError),
}

pub fn discover(path_or_id: &str) -> Result<ComponentHandle, LoadError> {
    if let Some(handle) = try_explicit(path_or_id)? {
        return Ok(handle);
    }
    if let Some(handle) = try_workspace(path_or_id)? {
        return Ok(handle);
    }
    if let Some(handle) = try_registry(path_or_id)? {
        return Ok(handle);
    }
    Err(LoadError::NotFound(path_or_id.to_string()))
}

fn try_explicit(arg: &str) -> Result<Option<ComponentHandle>, LoadError> {
    let path = Path::new(arg);
    if !path.exists() {
        return Ok(None);
    }

    let target = if path.is_dir() {
        path.join(MANIFEST_NAME)
    } else if path.extension().and_then(OsStr::to_str) == Some("json") {
        path.to_path_buf()
    } else if path.extension().and_then(OsStr::to_str) == Some("wasm") {
        path.parent()
            .map(|dir| dir.join(MANIFEST_NAME))
            .unwrap_or_else(|| path.to_path_buf())
    } else {
        path.join(MANIFEST_NAME)
    };

    if target.exists() {
        return load_from_manifest(&target).map(Some);
    }

    Ok(None)
}

fn try_workspace(id: &str) -> Result<Option<ComponentHandle>, LoadError> {
    let cwd = std::env::current_dir().map_err(|e| LoadError::Io {
        path: PathBuf::from("."),
        source: e,
    })?;
    let target = cwd.join("target").join("wasm32-wasip2");
    let file_name = format!("{id}.wasm");

    for profile in ["release", "debug"] {
        let candidate = target.join(profile).join(&file_name);
        if candidate.exists() {
            let manifest_path = candidate
                .parent()
                .map(|dir| dir.join(MANIFEST_NAME))
                .unwrap_or_else(|| candidate.with_extension("manifest.json"));
            if manifest_path.exists() {
                return load_from_manifest(&manifest_path).map(Some);
            }
        }
    }

    Ok(None)
}

fn try_registry(id: &str) -> Result<Option<ComponentHandle>, LoadError> {
    let Some(base) = BaseDirs::new() else {
        return Ok(None);
    };
    let registry_root = base.home_dir().join(".greentic").join("components");
    if !registry_root.exists() {
        return Ok(None);
    }

    let mut candidates = Vec::new();
    for entry in fs::read_dir(&registry_root).map_err(|err| LoadError::Io {
        path: registry_root.clone(),
        source: err,
    })? {
        let entry = entry.map_err(|err| LoadError::Io {
            path: registry_root.clone(),
            source: err,
        })?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name == id || (!id.contains('@') && name.starts_with(id)) {
            candidates.push(entry.path());
        }
    }

    candidates.sort();
    candidates.reverse();

    for dir in candidates {
        let manifest_path = dir.join(MANIFEST_NAME);
        if manifest_path.exists() {
            return load_from_manifest(&manifest_path).map(Some);
        }
    }

    Ok(None)
}

fn load_from_manifest(path: &Path) -> Result<ComponentHandle, LoadError> {
    let contents = fs::read_to_string(path).map_err(|source| LoadError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let manifest = parse_manifest(&contents).map_err(|source| LoadError::Manifest {
        path: path.to_path_buf(),
        source,
    })?;
    let root = path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let wasm_path = root.join(manifest.artifacts.component_wasm());
    if !wasm_path.exists() {
        return Err(LoadError::MissingArtifact { path: wasm_path });
    }
    verify_manifest_hash(&manifest, &root)?;
    Ok(ComponentHandle {
        manifest,
        wasm_path,
        root,
        manifest_path: path.to_path_buf(),
    })
}
