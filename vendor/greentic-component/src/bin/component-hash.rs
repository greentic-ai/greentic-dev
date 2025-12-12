#[cfg(feature = "cli")]
use anyhow::Result;
#[cfg(feature = "cli")]
use greentic_component::cmd::hash;

#[cfg(not(feature = "cli"))]
fn main() {
    eprintln!("component-hash requires the `cli` feature");
    std::process::exit(1);
}

#[cfg(feature = "cli")]
fn main() -> Result<()> {
    hash::run(hash::parse_from_cli())
}
