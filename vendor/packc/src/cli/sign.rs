#![forbid(unsafe_code)]

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use ed25519_dalek::pkcs8::DecodePrivateKey;
use ed25519_dalek::{Signer, SigningKey};
use greentic_types::{PackManifest, Signature, SignatureAlgorithm, encode_pack_manifest};

#[derive(Debug, Parser)]
pub struct SignArgs {
    /// Path to the pack directory containing pack.yaml
    #[arg(long = "pack", value_name = "DIR")]
    pub pack: PathBuf,

    /// Path to manifest.cbor (defaults to <pack>/dist/manifest.cbor)
    #[arg(long = "manifest", value_name = "FILE")]
    pub manifest: Option<PathBuf>,

    /// Ed25519 private key in PKCS#8 PEM format
    #[arg(long = "key", value_name = "FILE")]
    pub key: PathBuf,

    /// Optional key identifier to embed alongside the signature
    #[arg(long = "key-id", value_name = "ID", default_value = "default")]
    pub key_id: String,
}

pub fn handle(args: SignArgs, json: bool) -> Result<()> {
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

    let unsigned_bytes = encode_unsigned(&manifest)?;

    let private_pem = fs::read_to_string(&args.key)
        .with_context(|| format!("failed to read private key {}", args.key.display()))?;
    let signing_key =
        SigningKey::from_pkcs8_pem(&private_pem).context("failed to parse ed25519 private key")?;
    let signature_bytes = signing_key.sign(&unsigned_bytes).to_bytes().to_vec();

    let mut signed_manifest = manifest.clone();
    signed_manifest.signatures.signatures.push(Signature::new(
        args.key_id.clone(),
        SignatureAlgorithm::Ed25519,
        signature_bytes.clone(),
    ));
    let encoded =
        encode_pack_manifest(&signed_manifest).context("failed to encode signed manifest")?;

    fs::write(&manifest_path, &encoded)
        .with_context(|| format!("failed to write {}", manifest_path.display()))?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "status": "signed",
                "manifest": manifest_path,
                "key_id": args.key_id,
                "signatures": signed_manifest.signatures.signatures.len(),
            }))?
        );
    } else {
        println!(
            "signed manifest\n  manifest: {}\n  key_id: {}\n  signatures: {}",
            manifest_path.display(),
            args.key_id,
            signed_manifest.signatures.signatures.len()
        );
    }

    Ok(())
}

fn encode_unsigned(manifest: &PackManifest) -> Result<Vec<u8>> {
    let mut unsigned = manifest.clone();
    unsigned.signatures.signatures.clear();
    encode_pack_manifest(&unsigned).context("failed to encode unsigned manifest")
}
