use assert_cmd::prelude::*;
use ed25519_dalek::SigningKey;
use ed25519_dalek::pkcs8::{EncodePrivateKey, EncodePublicKey};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

#[test]
fn sign_and_verify_manifest() {
    let temp = tempdir().expect("temp dir");
    let manifest_out = temp.path().join("manifest.cbor");

    // Build manifest
    let mut build = Command::new(assert_cmd::cargo::cargo_bin!("packc"));
    build.current_dir(workspace_root());
    build.args([
        "build",
        "--in",
        "examples/weather-demo",
        "--manifest",
        manifest_out.to_str().unwrap(),
        "--log",
        "warn",
    ]);
    build.assert().success();

    // Generate keypair
    let signing_key = SigningKey::generate(&mut rand_core_06::OsRng);
    let priv_pem = signing_key
        .to_pkcs8_pem(pkcs8::LineEnding::LF)
        .expect("priv pem");
    let pub_pem = signing_key
        .verifying_key()
        .to_public_key_pem(pkcs8::LineEnding::LF)
        .expect("pub pem");

    let priv_path = temp.path().join("sk.pem");
    let pub_path = temp.path().join("pk.pem");
    fs::write(&priv_path, priv_pem.as_bytes()).expect("write sk");
    fs::write(&pub_path, pub_pem.as_bytes()).expect("write pk");

    // Sign
    let mut sign = Command::new(assert_cmd::cargo::cargo_bin!("packc"));
    sign.current_dir(workspace_root());
    sign.args([
        "sign",
        "--pack",
        "examples/weather-demo",
        "--manifest",
        manifest_out.to_str().unwrap(),
        "--key",
        priv_path.to_str().unwrap(),
        "--key-id",
        "test",
    ]);
    sign.assert().success();

    // Verify
    let mut verify = Command::new(assert_cmd::cargo::cargo_bin!("packc"));
    verify.current_dir(workspace_root());
    verify.args([
        "verify",
        "--pack",
        "examples/weather-demo",
        "--manifest",
        manifest_out.to_str().unwrap(),
        "--key",
        pub_path.to_str().unwrap(),
    ]);
    verify.assert().success();
}
