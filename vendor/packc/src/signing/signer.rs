#![forbid(unsafe_code)]

use std::path::Path;

use anyhow::{Result, anyhow};
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ed25519_dalek::Signer as _;
use ed25519_dalek::{SigningKey, pkcs8::DecodePrivateKey};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;

use crate::manifest::PackSignature;

use super::canon::{CanonicalizedPack, canonicalize_pack_dir};

/// Result of signing a pack directory.
pub struct SigningOutcome {
    /// Generated signature metadata.
    pub signature: PackSignature,
    /// Canonical bytes used for signing.
    pub canonical: CanonicalizedPack,
}

pub fn sign_pack(
    pack_dir: &Path,
    private_key_pem: &str,
    key_id_override: Option<&str>,
) -> Result<SigningOutcome> {
    let canonical = canonicalize_pack_dir(pack_dir)?;

    let signing_key = load_signing_key(private_key_pem)?;
    let verifying_key = signing_key.verifying_key();

    let key_id = key_id_override
        .map(|value| value.to_string())
        .unwrap_or_else(|| derive_key_id(verifying_key.as_bytes()));

    let signature = signing_key.sign(&canonical.bytes);
    let encoded_sig = URL_SAFE_NO_PAD.encode(signature.to_bytes());

    let pack_signature = PackSignature {
        alg: "ed25519".to_string(),
        key_id,
        created_at: OffsetDateTime::now_utc(),
        digest: format!("sha256:{}", canonical.digest_hex),
        sig: encoded_sig,
    };

    Ok(SigningOutcome {
        signature: pack_signature,
        canonical,
    })
}

fn load_signing_key(pem: &str) -> Result<SigningKey> {
    match SigningKey::from_pkcs8_pem(pem) {
        Ok(key) => Ok(key),
        Err(primary_err) => {
            // Support "BEGIN ED25519 PRIVATE KEY" by duck-typing the label.
            let (label, doc) = pkcs8::SecretDocument::from_pem(pem)
                .map_err(|err| anyhow!("failed to parse private key PEM: {err}"))?;

            if label != "ED25519 PRIVATE KEY" {
                return Err(anyhow!("unsupported private key format: {primary_err}"));
            }

            SigningKey::from_pkcs8_der(doc.as_bytes()).map_err(|err| {
                anyhow!("failed to load ED25519 private key from PKCS#8 data: {err}")
            })
        }
    }
}

fn derive_key_id(public_key_bytes: &[u8]) -> String {
    let digest = Sha256::digest(public_key_bytes);
    // Truncate to the first 16 bytes = 32 hex characters.
    hex::encode(&digest[..16])
}

// Re-export `SecretDocument` under this module to avoid leaking dependency internals.
mod pkcs8 {
    pub use pkcs8::SecretDocument;
}
