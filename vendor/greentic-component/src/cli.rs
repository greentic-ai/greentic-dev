use anyhow::{Error, Result, bail};
use clap::{Parser, Subcommand};

use crate::cmd::{
    self, build::BuildArgs, doctor::DoctorArgs, flow::FlowCommand, hash::HashArgs,
    inspect::InspectArgs, new::NewArgs, store::StoreCommand, templates::TemplatesArgs,
};
use crate::scaffold::engine::ScaffoldEngine;

#[derive(Parser, Debug)]
#[command(
    name = "greentic-component",
    about = "Toolkit for Greentic component developers",
    version,
    propagate_version = true,
    arg_required_else_help = true,
    disable_version_flag = true
)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Scaffold a new Greentic component project
    New(NewArgs),
    /// List available component templates
    Templates(TemplatesArgs),
    /// Run component doctor checks
    Doctor(DoctorArgs),
    /// Inspect manifests and describe payloads
    Inspect(InspectArgs),
    /// Recompute manifest hashes
    Hash(HashArgs),
    /// Build component wasm + scaffold config flows
    Build(BuildArgs),
    /// Flow utilities (config flow scaffolding)
    #[command(subcommand)]
    Flow(FlowCommand),
    /// Interact with the component store
    #[command(subcommand)]
    Store(StoreCommand),
}

pub fn main() -> Result<()> {
    let cli = Cli::parse();
    let engine = ScaffoldEngine::new();
    match cli.command {
        Commands::New(args) => cmd::new::run(args, &engine),
        Commands::Templates(args) => cmd::templates::run(args, &engine),
        Commands::Doctor(args) => cmd::doctor::run(args).map_err(Error::new),
        Commands::Inspect(args) => {
            let result = cmd::inspect::run(&args)?;
            cmd::inspect::emit_warnings(&result.warnings);
            if args.strict && !result.warnings.is_empty() {
                bail!(
                    "component-inspect: {} warning(s) treated as errors (--strict)",
                    result.warnings.len()
                );
            }
            Ok(())
        }
        Commands::Hash(args) => cmd::hash::run(args),
        Commands::Build(args) => cmd::build::run(args),
        Commands::Flow(flow_cmd) => cmd::flow::run(flow_cmd),
        Commands::Store(store_cmd) => cmd::store::run(store_cmd),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_new_subcommand() {
        let cli = Cli::try_parse_from(["greentic-component", "new", "--name", "demo", "--json"])
            .expect("expected CLI to parse");
        match cli.command {
            Commands::New(args) => {
                assert_eq!(args.name, "demo");
                assert!(args.json);
                assert!(!args.no_check);
                assert!(!args.no_git);
            }
            _ => panic!("expected new args"),
        }
    }
}
