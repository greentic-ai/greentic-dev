#![cfg(feature = "cli")]

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use tempfile::NamedTempFile;
use thiserror::Error;

#[derive(Debug, Default, Clone)]
pub struct Writer;

impl Writer {
    pub fn new() -> Self {
        Self
    }

    pub fn write_all(
        &self,
        root: &Path,
        files: &[GeneratedFile],
    ) -> Result<Vec<String>, WriteError> {
        if !root.exists() {
            fs::create_dir_all(root)
                .map_err(|err| WriteError::CreateDir(root.to_path_buf(), err))?;
        }

        let mut created = Vec::with_capacity(files.len());
        for file in files {
            let target = root.join(&file.relative_path);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)
                    .map_err(|err| WriteError::CreateDir(parent.to_path_buf(), err))?;
            }

            let mut tmp = NamedTempFile::new_in(target.parent().unwrap_or(root))
                .map_err(|err| WriteError::WriteFile(file.relative_path.to_path_buf(), err))?;
            tmp.write_all(&file.contents)
                .map_err(|err| WriteError::WriteFile(file.relative_path.to_path_buf(), err))?;
            tmp.flush()
                .map_err(|err| WriteError::WriteFile(file.relative_path.to_path_buf(), err))?;
            tmp.as_file()
                .sync_all()
                .map_err(|err| WriteError::WriteFile(file.relative_path.to_path_buf(), err))?;

            tmp.persist(&target)
                .map_err(|err| WriteError::Persist(file.relative_path.to_path_buf(), err.error))?;

            if file.executable {
                set_executable(&target).map_err(|err| {
                    WriteError::Permissions(file.relative_path.to_path_buf(), err)
                })?;
            }

            created.push(file.relative_path.display().to_string());
        }

        created.sort();
        Ok(created)
    }
}

fn set_executable(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(path)?;
        let mut permissions = metadata.permissions();
        let current = permissions.mode();
        permissions.set_mode(current | 0o755);
        fs::set_permissions(path, permissions)
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct GeneratedFile {
    pub relative_path: PathBuf,
    pub contents: Vec<u8>,
    pub executable: bool,
}

#[derive(Debug, Error)]
pub enum WriteError {
    #[error("failed to create directory {0}: {1}")]
    CreateDir(PathBuf, #[source] io::Error),
    #[error("failed to write {0}: {1}")]
    WriteFile(PathBuf, #[source] io::Error),
    #[error("failed to persist {0}: {1}")]
    Persist(PathBuf, #[source] io::Error),
    #[error("failed to update permissions for {0}: {1}")]
    Permissions(PathBuf, #[source] io::Error),
}
