#[cfg(feature = "schema")]
fn main() -> anyhow::Result<()> {
    use std::{fs, path::PathBuf};

    let schemas_dir = PathBuf::from("dist/schemas/v1");
    greentic_types::write_all_schemas(&schemas_dir)?;

    let dist_root = PathBuf::from("dist");
    fs::create_dir_all(&dist_root)?;
    fs::write(dist_root.join(".nojekyll"), [])?;

    println!("Schemas exported to {}", schemas_dir.display());
    Ok(())
}

#[cfg(not(feature = "schema"))]
fn main() -> anyhow::Result<()> {
    anyhow::bail!("enable the `schema` feature to run export-schemas")
}
