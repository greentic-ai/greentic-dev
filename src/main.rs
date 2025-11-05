use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use dev_runner::{
    ComponentSchema, DescribeRegistry, FlowTranscript, FlowValidator, StaticComponentDescriber,
    TranscriptStore, schema_id_from_json,
};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run(args) => run_flow(args),
    }
}

fn run_flow(args: RunArgs) -> Result<()> {
    let registry = DescribeRegistry::new();

    if args.print_schemas {
        println!("Known component schemas:");
        for (name, stub) in registry.iter() {
            println!(" - {name}");
            if let Some(schema_id) = schema_id_from_json(&stub.schema) {
                println!("   schema id: {schema_id}");
            }

            let defaults = serde_yaml_bw::to_string(&stub.defaults)?;
            let defaults = defaults.trim();
            if !defaults.is_empty() {
                println!("   defaults:");
                for line in defaults.lines() {
                    println!("     {line}");
                }
            }
        }
    }

    let describer = StaticComponentDescriber::new().with_fallback(ComponentSchema {
        node_schema: Some(r#"{"type":"object"}"#.to_owned()),
    });

    // Components can register specific schemas here once describe() wiring is available.
    let validator = FlowValidator::new(describer, registry);

    let validated_nodes = validator.validate_file(&args.file)?;

    if args.validate_only {
        println!("Schema validation succeeded for `{}`", args.file.display());
        return Ok(());
    }

    let transcript = FlowTranscript::from_validated_nodes(&args.file, &validated_nodes);
    let store = TranscriptStore::default();
    let transcript_path = store.write_transcript(&args.file, &transcript)?;

    println!(
        "Schema validation succeeded for `{}` (flow execution not yet implemented).",
        args.file.display()
    );
    println!("Transcript stored at `{}`", transcript_path.display());

    Ok(())
}

#[derive(Parser)]
#[command(name = "greentic-dev")]
#[command(version)]
#[command(about = "Greentic developer tooling CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run(RunArgs),
}

#[derive(Args)]
struct RunArgs {
    /// Path to the flow YAML file
    #[arg(short = 'f', long = "file")]
    file: PathBuf,

    /// Only run schema validation without executing the flow
    #[arg(long = "validate-only")]
    validate_only: bool,

    /// Print the stub schemas known to the registry
    #[arg(long = "print-schemas")]
    print_schemas: bool,
}
