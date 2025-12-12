#![forbid(unsafe_code)]

use std::ffi::OsStr;
use std::fs;
use std::path::{Component, Path};

use anyhow::{Context, Result, anyhow};
use ignore::WalkBuilder;
use sha2::{Digest, Sha256};

use crate::manifest;

/// Canonical representation of a pack directory used for signing.
pub struct CanonicalizedPack {
    /// Concatenated canonical bytes over which the signature is produced.
    pub bytes: Vec<u8>,
    /// Hex encoded SHA-256 digest of the canonical bytes.
    pub digest_hex: String,
}

/// Computes the canonical byte stream of the provided pack directory.
pub fn canonicalize_pack_dir(pack_dir: &Path) -> Result<CanonicalizedPack> {
    let pack_dir = pack_dir
        .canonicalize()
        .with_context(|| format!("failed to resolve pack directory {}", pack_dir.display()))?;

    let mut entries = Vec::new();

    let mut builder = WalkBuilder::new(&pack_dir);
    builder
        .standard_filters(false)
        .git_ignore(false)
        .git_exclude(false)
        .git_global(false)
        .hidden(false)
        .follow_links(true);

    builder.add_custom_ignore_filename(".packignore");

    let walker = builder.build();

    for entry in walker {
        let entry = entry.with_context(|| "failed to walk pack directory")?;

        if entry.depth() == 0 {
            continue;
        }

        let file_type = match entry.file_type() {
            Some(ft) => ft,
            None => continue,
        };

        if file_type.is_dir() {
            continue;
        }

        let abs_path = entry.path();
        let rel_path = abs_path
            .strip_prefix(&pack_dir)
            .with_context(|| format!("failed to strip prefix for {}", abs_path.display()))?;

        if should_skip(rel_path) {
            continue;
        }

        let rel_path_str = normalize_path(rel_path)
            .ok_or_else(|| anyhow!("path {} is not valid UTF-8", rel_path.display()))?;

        let contents = if manifest::is_pack_manifest_path(rel_path) {
            manifest::read_manifest_without_signature(abs_path)?
        } else {
            fs::read(abs_path).with_context(|| format!("failed to read {}", abs_path.display()))?
        };

        entries.push(CanonicalEntry {
            rel_path: rel_path_str,
            contents,
        });
    }

    entries.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));

    let mut buffer = Vec::new();
    for entry in &entries {
        let header = format!("PATH\0{}\nLEN\0{}\n", entry.rel_path, entry.contents.len());
        buffer.extend_from_slice(header.as_bytes());
        buffer.extend_from_slice(&entry.contents);
    }

    let digest = Sha256::digest(&buffer);
    let digest_hex = hex::encode(digest);

    Ok(CanonicalizedPack {
        bytes: buffer,
        digest_hex,
    })
}

struct CanonicalEntry {
    rel_path: String,
    contents: Vec<u8>,
}

fn should_skip(path: &Path) -> bool {
    if path.components().any(|component| match component {
        Component::Normal(name) => matches!(name.to_str(), Some(".git") | Some("target")),
        _ => false,
    }) {
        return true;
    }

    matches!(path.file_name().and_then(OsStr::to_str), Some(".DS_Store"))
}

fn normalize_path(path: &Path) -> Option<String> {
    let mut segments = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(seg) => segments.push(seg.to_str()?.to_string()),
            Component::CurDir => {}
            Component::ParentDir => segments.push("..".to_string()),
            Component::Prefix(_) | Component::RootDir => return None,
        }
    }

    Some(segments.join("/"))
}
