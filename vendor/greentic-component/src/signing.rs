use std::fs;
use std::path::Path;

use blake3::Hasher;
use thiserror::Error;

use crate::manifest::ComponentManifest;

#[derive(Debug, Clone, Default)]
pub struct DevPolicy {
    pub allow_missing_signatures: bool,
}

#[derive(Debug, Clone, Default)]
pub struct StrictPolicy {
    pub signatures: Vec<SignatureRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureRef {
    pub issuer: String,
    pub digest: String,
}

#[derive(Debug, Error)]
pub enum SigningError {
    #[error("failed to read component bytes: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },
    #[error("component hash mismatch (expected {expected}, found {found})")]
    HashMismatch { expected: String, found: String },
}

pub fn verify_manifest_hash(manifest: &ComponentManifest, root: &Path) -> Result<(), SigningError> {
    let wasm_path = root.join(manifest.artifacts.component_wasm());
    verify_wasm_hash(manifest.hashes.component_wasm.as_str(), &wasm_path)
}

pub fn verify_wasm_hash(expected: &str, wasm_path: &Path) -> Result<(), SigningError> {
    let actual = compute_wasm_hash(wasm_path)?;
    if actual != expected {
        return Err(SigningError::HashMismatch {
            expected: expected.to_string(),
            found: actual,
        });
    }
    Ok(())
}

pub fn compute_wasm_hash(wasm_path: &Path) -> Result<String, SigningError> {
    let mut hasher = Hasher::new();
    let bytes = fs::read(wasm_path)?;
    hasher.update(&bytes);
    let digest = hasher.finalize();
    Ok(format!("blake3:{}", hex::encode(digest.as_bytes())))
}
