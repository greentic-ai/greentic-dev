use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand};

use crate::store::{CompatPolicy, ComponentStore};

#[derive(Subcommand, Debug, Clone)]
pub enum StoreCommand {
    /// Fetch a component from a source and write the wasm bytes to disk
    Fetch(StoreFetchArgs),
}

#[derive(Args, Debug, Clone)]
pub struct StoreFetchArgs {
    /// Filesystem path to a component directory containing component.manifest.json
    #[arg(long, value_name = "PATH", conflicts_with = "oci")]
    pub fs: Option<PathBuf>,
    /// OCI reference to a published component (requires the `oci` feature)
    #[arg(long, value_name = "REF", conflicts_with = "fs")]
    pub oci: Option<String>,
    /// Destination path to write the fetched component bytes
    #[arg(long, value_name = "PATH")]
    pub output: PathBuf,
    /// Optional cache directory for fetched components
    #[arg(long, value_name = "DIR")]
    pub cache_dir: Option<PathBuf>,
    /// Source identifier to register inside the store
    #[arg(long, value_name = "ID", default_value = "default")]
    pub source: String,
}

pub fn run(command: StoreCommand) -> Result<()> {
    match command {
        StoreCommand::Fetch(args) => fetch(args),
    }
}

fn fetch(args: StoreFetchArgs) -> Result<()> {
    let mut store = ComponentStore::with_cache_dir(args.cache_dir.clone(), CompatPolicy::default());
    match (&args.fs, &args.oci) {
        (Some(path), None) => {
            store.add_fs(&args.source, path);
        }
        (None, Some(reference)) => {
            store.add_oci(&args.source, reference);
        }
        _ => bail!("specify exactly one of --fs or --oci"),
    }
    let rt = tokio::runtime::Runtime::new().context("failed to create async runtime")?;
    let bytes = rt
        .block_on(async { store.get(&args.source).await })
        .context("store fetch failed")?;
    fs::write(&args.output, &bytes.bytes)?;
    println!(
        "Wrote {} ({} bytes) for source {}",
        args.output.display(),
        bytes.bytes.len(),
        args.source
    );
    Ok(())
}
