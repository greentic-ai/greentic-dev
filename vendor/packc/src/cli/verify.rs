#![forbid(unsafe_code)]

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use ed25519_dalek::VerifyingKey;
use ed25519_dalek::pkcs8::DecodePublicKey;
use greentic_types::{PackManifest, SignatureAlgorithm, encode_pack_manifest};

#[derive(Debug, Parser)]
pub struct VerifyArgs {
    /// Path to the pack directory containing pack.yaml
    #[arg(long = "pack", value_name = "DIR")]
    pub pack: PathBuf,

    /// Path to manifest.cbor (defaults to <pack>/dist/manifest.cbor)
    #[arg(long = "manifest", value_name = "FILE")]
    pub manifest: Option<PathBuf>,

    /// Ed25519 public key in PKCS#8 PEM format
    #[arg(long = "key", value_name = "FILE")]
    pub key: PathBuf,
}

pub fn handle(args: VerifyArgs, json: bool) -> Result<()> {
    let pack_dir = args
        .pack
        .canonicalize()
        .with_context(|| format!("failed to resolve pack dir {}", args.pack.display()))?;
    let manifest_path = args
        .manifest
        .map(|p| if p.is_relative() { pack_dir.join(p) } else { p })
        .unwrap_or_else(|| pack_dir.join("dist").join("manifest.cbor"));

    let manifest_bytes = fs::read(&manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let manifest: PackManifest = greentic_types::decode_pack_manifest(&manifest_bytes)
        .context("manifest.cbor is not a valid PackManifest")?;

    if manifest.signatures.signatures.is_empty() {
        anyhow::bail!("no signatures present in manifest");
    }

    let public_pem = fs::read_to_string(&args.key)
        .with_context(|| format!("failed to read public key {}", args.key.display()))?;
    let verifying_key =
        VerifyingKey::from_public_key_pem(&public_pem).context("failed to parse public key")?;

    let unsigned_bytes = encode_unsigned(&manifest)?;

    let mut verified = false;
    let mut errors = Vec::new();
    for sig in &manifest.signatures.signatures {
        if sig.algorithm != SignatureAlgorithm::Ed25519 {
            errors.push(format!("unsupported algorithm {:?}", sig.algorithm));
            continue;
        }
        let Ok(signature) = ed25519_dalek::Signature::try_from(sig.signature.as_slice()) else {
            errors.push("invalid signature bytes".to_string());
            continue;
        };
        if verifying_key
            .verify_strict(&unsigned_bytes, &signature)
            .is_ok()
        {
            verified = true;
            break;
        } else {
            errors.push("signature verification failed".to_string());
        }
    }

    if !verified {
        anyhow::bail!("no signatures verified: {}", errors.join(", "));
    }

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "status": "verified",
                "manifest": manifest_path,
                "signatures": manifest.signatures.signatures.len(),
            }))?
        );
    } else {
        println!(
            "verified manifest\n  manifest: {}\n  signatures checked: {}",
            manifest_path.display(),
            manifest.signatures.signatures.len()
        );
    }

    Ok(())
}

fn encode_unsigned(manifest: &PackManifest) -> Result<Vec<u8>> {
    let mut unsigned = manifest.clone();
    unsigned.signatures.signatures.clear();
    encode_pack_manifest(&unsigned).context("failed to encode unsigned manifest")
}
