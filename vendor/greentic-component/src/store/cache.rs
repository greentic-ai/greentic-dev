use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bytes::Bytes;
use directories::BaseDirs;
use sha2::{Digest, Sha256};
use tokio::fs as tfs;
use tokio::io::AsyncWriteExt;

use super::{ComponentBytes, ComponentId, ComponentLocator, meta};
use crate::path_safety::normalize_under_root;

#[derive(Clone, Debug)]
pub struct Cache {
    root: PathBuf,
}

impl Default for Cache {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Cache {
    pub fn new(user_specified: Option<PathBuf>) -> Self {
        let root = user_specified.unwrap_or_else(|| {
            BaseDirs::new()
                .map(|dirs| dirs.home_dir().join(".greentic").join("components"))
                .unwrap_or_else(|| PathBuf::from(".greentic").join("components"))
        });
        Cache { root }
    }

    async fn entry_path_for_id(&self, id: &ComponentId) -> Result<PathBuf> {
        let sanitized = id.0.replace(':', "_");
        self.normalize_in_root(Path::new(&sanitized)).await
    }

    fn key_for_locator(loc: &ComponentLocator) -> String {
        match loc {
            ComponentLocator::Fs { path } => format!("fs:{}", path.to_string_lossy()),
            ComponentLocator::Oci { reference } => format!("oci:{reference}"),
        }
    }

    async fn hint_path_for_locator(&self, loc: &ComponentLocator) -> Result<PathBuf> {
        let mut hasher = Sha256::new();
        hasher.update(Self::key_for_locator(loc));
        let digest = hex::encode(hasher.finalize());
        let candidate = PathBuf::from("_loc").join(digest);
        self.normalize_in_root(&candidate).await
    }

    async fn normalize_in_root(&self, candidate: &Path) -> Result<PathBuf> {
        tfs::create_dir_all(&self.root)
            .await
            .with_context(|| format!("unable to create cache root at {}", self.root.display()))?;
        normalize_under_root(&self.root, candidate)
    }

    pub async fn try_load(&self, loc: &ComponentLocator) -> Result<Option<ComponentBytes>> {
        let hint_path = self.hint_path_for_locator(loc).await?;
        if !path_exists(&hint_path).await {
            return Ok(None);
        }

        let id_hex = tfs::read_to_string(&hint_path).await?;
        let id = ComponentId(id_hex.trim().to_owned());
        let data_path = self.entry_path_for_id(&id).await?;
        if !path_exists(&data_path).await {
            return Ok(None);
        }

        let bytes_vec = tfs::read(&data_path).await?;
        let (computed_id, meta) = meta::compute_id_and_meta(&bytes_vec).await?;
        let bytes = Bytes::from(bytes_vec);

        // Update hint if the stored id mismatched (e.g., manual tampering).
        if computed_id != id {
            self.write_hint(loc, &computed_id).await?;
        }

        Ok(Some(ComponentBytes {
            id: computed_id,
            bytes,
            meta,
        }))
    }

    pub async fn store(&self, loc: &ComponentLocator, cb: &ComponentBytes) -> Result<()> {
        tfs::create_dir_all(&self.root)
            .await
            .with_context(|| format!("unable to create cache root at {}", self.root.display()))?;

        let path = self.entry_path_for_id(&cb.id).await?;
        if let Some(parent) = path.parent() {
            tfs::create_dir_all(parent).await?;
        }
        let mut file = tfs::File::create(&path).await?;
        file.write_all(cb.bytes.as_ref()).await?;
        file.flush().await?;

        self.write_hint(loc, &cb.id).await?;
        Ok(())
    }

    async fn write_hint(&self, loc: &ComponentLocator, id: &ComponentId) -> Result<()> {
        let hint_path = self.hint_path_for_locator(loc).await?;
        if let Some(parent) = hint_path.parent() {
            tfs::create_dir_all(parent).await?;
        }
        tfs::write(&hint_path, id.0.as_bytes()).await?;
        Ok(())
    }
}

async fn path_exists(path: &Path) -> bool {
    tfs::try_exists(path).await.unwrap_or(false)
}
