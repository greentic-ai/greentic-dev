use std::path::Path;

use anyhow::Result;
use bytes::Bytes;
use tokio::fs as tfs;

use crate::path_safety::normalize_under_root;

pub async fn fetch(root: &Path, path: &Path) -> Result<Bytes> {
    let safe = normalize_under_root(root, path)?;
    let data = tfs::read(&safe).await?;
    Ok(Bytes::from(data))
}
