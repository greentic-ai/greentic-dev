#![forbid(unsafe_code)]

use std::path::Path;

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ed25519_dalek::Verifier as _;
use ed25519_dalek::{Signature as Ed25519Signature, VerifyingKey, pkcs8::DecodePublicKey};
use sha2::{Digest, Sha256};
use thiserror::Error;
use time::OffsetDateTime;

use crate::manifest::{self, PackSignature};

use super::{VerifyOptions, canonicalize_pack_dir};

/// Errors that may occur while verifying a pack signature.
#[derive(Debug, Error)]
pub enum VerificationError {
    #[error("pack manifest is missing a greentic.signature block")]
    MissingSignature,
    #[error("computed digest {computed} does not match manifest digest {expected}")]
    DigestMismatch { expected: String, computed: String },
    #[error("signature algorithm {algorithm} is not supported")]
    UnsupportedAlgorithm { algorithm: String },
    #[error("public key not provided for key id {key_id}")]
    KeyNotFound { key_id: String },
    #[error("public key does not match manifest key id (expected {expected}, got {provided})")]
    KeyIdMismatch { expected: String, provided: String },
    #[error("failed to decode signature: {0}")]
    SignatureDecode(#[from] base64::DecodeError),
    #[error("signature has invalid length: {0}")]
    SignatureLength(usize),
    #[error("failed to parse public key PEM: {0}")]
    PublicKey(#[from] pkcs8::Error),
    #[error("failed to parse public key (SPKI): {0}")]
    PublicKeySpki(#[from] pkcs8::spki::Error),
    #[error("signature verification failed for key {key_id}")]
    InvalidSignature { key_id: String },
    #[error("signature bytes were malformed")]
    SignatureMalformed,
    #[error("manifest error: {0}")]
    Manifest(#[from] anyhow::Error),
}

/// Verifies a signed pack directory.
pub fn verify_pack(
    pack_dir: &Path,
    opts: VerifyOptions<'_>,
) -> Result<PackSignature, VerificationError> {
    let canonical = canonicalize_pack_dir(pack_dir).map_err(VerificationError::Manifest)?;

    let signature_opt = manifest::read_signature(pack_dir).map_err(VerificationError::Manifest)?;

    let Some(signature) = signature_opt else {
        if opts.allow_unsigned {
            return Ok(PackSignature {
                alg: "none".to_string(),
                key_id: "unsigned".to_string(),
                created_at: OffsetDateTime::UNIX_EPOCH,
                digest: format!("sha256:{}", canonical.digest_hex),
                sig: String::new(),
            });
        }

        return Err(VerificationError::MissingSignature);
    };

    if !signature.alg.eq_ignore_ascii_case("ed25519") {
        return Err(VerificationError::UnsupportedAlgorithm {
            algorithm: signature.alg.clone(),
        });
    }

    let expected_digest = format!("sha256:{}", canonical.digest_hex);
    if signature.digest != expected_digest {
        return Err(VerificationError::DigestMismatch {
            expected: signature.digest.clone(),
            computed: expected_digest,
        });
    }

    let public_key_pem = opts
        .public_key_pem
        .ok_or_else(|| VerificationError::KeyNotFound {
            key_id: signature.key_id.clone(),
        })?;

    let verifying_key = VerifyingKey::from_public_key_pem(public_key_pem)
        .map_err(VerificationError::PublicKeySpki)?;
    let derived_key_id = derive_key_id(verifying_key.as_bytes());
    if derived_key_id != signature.key_id {
        return Err(VerificationError::KeyIdMismatch {
            expected: signature.key_id.clone(),
            provided: derived_key_id,
        });
    }

    let raw_signature = URL_SAFE_NO_PAD.decode(signature.sig.as_bytes())?;
    if raw_signature.len() != Ed25519Signature::BYTE_SIZE {
        return Err(VerificationError::SignatureLength(raw_signature.len()));
    }

    let ed_signature = Ed25519Signature::from_slice(&raw_signature)
        .map_err(|_| VerificationError::SignatureMalformed)?;

    verifying_key
        .verify(&canonical.bytes, &ed_signature)
        .map_err(|_| VerificationError::InvalidSignature {
            key_id: signature.key_id.clone(),
        })?;

    Ok(signature)
}

fn derive_key_id(public_key_bytes: &[u8]) -> String {
    let digest = Sha256::digest(public_key_bytes);
    hex::encode(&digest[..16])
}
