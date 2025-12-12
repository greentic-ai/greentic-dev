use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Normalize a user-supplied path and ensure it stays within an allowed root.
/// Reject absolute paths and any that escape via `..`.
pub fn normalize_under_root(root: &Path, candidate: &Path) -> Result<PathBuf> {
    if candidate.is_absolute() {
        anyhow::bail!("absolute paths are not allowed: {}", candidate.display());
    }

    let root = root
        .canonicalize()
        .with_context(|| format!("failed to canonicalize root {}", root.display()))?;

    // Join and canonicalize. If the full path does not exist yet (e.g., we are
    // about to create it), canonicalize the nearest existing ancestor instead.
    let joined = root.join(candidate);
    let canon =
        match joined.canonicalize() {
            Ok(path) => path,
            Err(err) if err.kind() == ErrorKind::NotFound => {
                let mut missing: Vec<PathBuf> = Vec::new();
                let mut ancestor = joined.as_path();
                loop {
                    if ancestor.try_exists().unwrap_or(false) {
                        break;
                    }
                    let parent = ancestor
                        .parent()
                        .with_context(|| format!("{} has no parent", ancestor.display()))?;
                    missing.push(ancestor.file_name().map(PathBuf::from).with_context(|| {
                        format!("{} missing final component", ancestor.display())
                    })?);
                    ancestor = parent;
                }

                let mut rebuilt = ancestor
                    .canonicalize()
                    .with_context(|| format!("failed to canonicalize {}", ancestor.display()))?;
                while let Some(component) = missing.pop() {
                    rebuilt.push(component);
                }
                rebuilt
            }
            Err(err) => {
                return Err(err)
                    .with_context(|| format!("failed to canonicalize {}", joined.display()));
            }
        };

    // Ensure the canonical path is still under root
    if !canon.starts_with(&root) {
        anyhow::bail!(
            "path escapes root ({}): {}",
            root.display(),
            canon.display()
        );
    }

    Ok(canon)
}
