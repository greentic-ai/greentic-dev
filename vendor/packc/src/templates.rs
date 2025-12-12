use crate::manifest::PackSpec;
use crate::path_safety::normalize_under_root;
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct TemplateAsset {
    pub logical_path: String,
    #[allow(dead_code)]
    pub absolute_path: PathBuf,
    pub bytes: Vec<u8>,
    pub sha256: String,
    pub size: u64,
}

pub fn collect_templates(pack_dir: &Path, spec: &PackSpec) -> Result<Vec<TemplateAsset>> {
    let mut assets = Vec::new();
    let mut seen_paths = BTreeSet::new();

    for dir in &spec.template_dirs {
        let relative_dir = PathBuf::from(dir);
        let absolute_dir = normalize_under_root(pack_dir, &relative_dir)?;

        if !absolute_dir.exists() {
            tracing::warn!("template directory missing: {}", absolute_dir.display());
            continue;
        }

        for entry in WalkDir::new(&absolute_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.into_path();
            let rel_to_root = path
                .strip_prefix(&absolute_dir)
                .unwrap_or(path.as_path())
                .to_path_buf();
            let logical_path = relative_dir
                .join(&rel_to_root)
                .components()
                .map(|comp| comp.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");

            if !seen_paths.insert(logical_path.clone()) {
                anyhow::bail!("duplicate template asset detected: {}", logical_path);
            }

            let bytes = fs::read(&path)
                .with_context(|| format!("failed to read template {}", path.display()))?;
            let digest = Sha256::digest(&bytes);
            let sha256 = hex::encode(digest);
            let size = bytes.len() as u64;

            assets.push(TemplateAsset {
                logical_path,
                absolute_path: path,
                bytes,
                sha256,
                size,
            });
        }
    }

    assets.sort_by(|a, b| a.logical_path.cmp(&b.logical_path));
    Ok(assets)
}
