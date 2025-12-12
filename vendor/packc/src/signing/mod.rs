#![forbid(unsafe_code)]

use std::path::Path;

use anyhow::Result;

use crate::manifest::{self, PackSignature};

pub mod canon;
pub mod signer;
pub mod verify;

pub use canon::{CanonicalizedPack, canonicalize_pack_dir};
pub use verify::VerificationError;

/// Options used when verifying pack signatures.
#[derive(Debug, Clone, Copy, Default)]
pub struct VerifyOptions<'a> {
    /// Public key in PEM format. When absent, signatures cannot be validated.
    pub public_key_pem: Option<&'a str>,
    /// Allow manifests without signatures.
    pub allow_unsigned: bool,
}

/// Signs a pack directory using the provided private key and embeds the signature
/// into the manifest.
pub fn sign_pack_dir(
    pack_dir: &Path,
    private_key_pem: &str,
    key_id: Option<&str>,
) -> Result<PackSignature> {
    let outcome = signer::sign_pack(pack_dir, private_key_pem, key_id)?;
    manifest::write_signature(pack_dir, &outcome.signature, None)?;
    Ok(outcome.signature)
}

/// Verifies a pack directory using the supplied options.
pub fn verify_pack_dir(pack_dir: &Path, opts: VerifyOptions<'_>) -> Result<PackSignature> {
    verify::verify_pack(pack_dir, opts).map_err(anyhow::Error::new)
}
