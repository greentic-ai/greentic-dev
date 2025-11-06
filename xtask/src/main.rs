use clap::Parser;
use greentic_dev::component_cli::{ComponentCommands, run_component_command};

#[derive(Parser)]
#[command(name = "xtask")]
#[command(version)]
#[command(about = "Developer tooling tasks for the Greentic workspace")]
struct Cli {
    #[command(subcommand)]
    command: ComponentCommands,
}

fn main() {
    let cli = Cli::parse();

    if let Err(error) = run_component_command(cli.command) {
        eprintln!("{error:?}");
        std::process::exit(1);
    }
}
