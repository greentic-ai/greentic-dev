use anyhow::{Result, anyhow};
use bytes::Bytes;
use oci_distribution::Reference;
use oci_distribution::client::{Client, ClientConfig};
use oci_distribution::secrets::RegistryAuth;

pub async fn fetch(reference: &str) -> Result<Bytes> {
    let reference: Reference = reference
        .parse()
        .map_err(|err| anyhow!("invalid OCI reference '{reference}': {err}"))?;

    let client = Client::new(ClientConfig::default());
    let auth = RegistryAuth::Anonymous;
    let accepted_media_types = vec![
        "application/wasm",
        "application/octet-stream",
        "application/vnd.module.wasm.content.layer.v1+wasm",
        "application/vnd.module.wasm.content.layer.v1+tar",
    ];
    let image = client.pull(&reference, &auth, accepted_media_types).await?;

    let mut selected = None;
    for layer in image.layers.into_iter() {
        let media_type = layer.media_type.to_ascii_lowercase();
        if media_type.contains("application/wasm") || media_type.contains("octet-stream") {
            selected = Some(layer.data);
            break;
        }
    }

    let bytes =
        selected.ok_or_else(|| anyhow!("no suitable component layer found for {reference}"))?;
    Ok(Bytes::from(bytes))
}
