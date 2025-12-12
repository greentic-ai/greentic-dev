use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result};

/// Normalise a user-supplied path so it stays under `root`.
/// Rejects absolute inputs and any traversal that would escape `root`.
pub fn normalize_under_root(root: &Path, candidate: &Path) -> Result<PathBuf> {
    if candidate.is_absolute() {
        anyhow::bail!("absolute paths are not allowed: {}", candidate.display());
    }

    let canon_root = root
        .canonicalize()
        .with_context(|| format!("failed to canonicalize root {}", root.display()))?;
    let mut normalized = canon_root.clone();
    let root_depth = canon_root.components().count();

    for comp in candidate.components() {
        match comp {
            Component::CurDir => {}
            Component::Normal(part) => normalized.push(part),
            Component::ParentDir => {
                if root_depth == 0 || !normalized.pop() {
                    anyhow::bail!(
                        "path escapes root ({}): {}",
                        canon_root.display(),
                        candidate.display()
                    );
                }
                // Prevent walking above the canonical root depth.
                if normalized.components().count() < root_depth {
                    anyhow::bail!(
                        "path escapes root ({}): {}",
                        canon_root.display(),
                        candidate.display()
                    );
                }
            }
            Component::Prefix(_) | Component::RootDir => {
                anyhow::bail!("invalid path component in {}", candidate.display());
            }
        }
    }

    if !normalized.starts_with(&canon_root) {
        anyhow::bail!(
            "path escapes root ({}): {}",
            canon_root.display(),
            candidate.display()
        );
    }

    Ok(normalized)
}
