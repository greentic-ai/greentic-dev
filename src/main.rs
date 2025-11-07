mod component_resolver;
mod flow_cmd;
#[cfg(feature = "mcp")]
mod mcp_cmd;
mod pack_build;
mod pack_run;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::pack_build::PackSigning;
use crate::pack_run::{MockSetting, RunPolicy};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Flow(flow) => match flow {
            FlowCommand::Validate(args) => flow_cmd::validate(&args.file, args.json),
        },
        Command::Pack(pack) => match pack {
            PackCommand::Build(args) => pack_build::run(
                &args.file,
                &args.out,
                args.sign.into(),
                args.meta.as_deref(),
                args.component_dir.as_deref(),
            ),
            PackCommand::Run(args) => {
                let allow_hosts = args
                    .allow
                    .as_ref()
                    .map(|value| split_allow_list(value))
                    .transpose()?;
                pack_run::run(pack_run::PackRunConfig {
                    pack_path: &args.pack,
                    entry: args.entry,
                    input: args.input,
                    policy: args.policy.into(),
                    otlp: args.otlp,
                    allow_hosts,
                    mocks: args.mocks.into(),
                    artifacts_dir: args.artifacts.as_deref(),
                })
            }
        },
        Command::Component(component) => match component {
            ComponentCommand::Inspect(args) => component_resolver::inspect(&args.target, args.json),
            ComponentCommand::Doctor(args) => component_resolver::doctor(&args.target),
        },
        #[cfg(feature = "mcp")]
        Command::Mcp(mcp) => match mcp {
            McpCommand::Doctor(args) => mcp_cmd::doctor(&args.provider, args.json),
        },
    }
}

#[derive(Parser)]
#[command(name = "greentic-dev")]
#[command(version)]
#[command(about = "Greentic developer tooling CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Flow tooling (validate, lint, bundle inspection)
    #[command(subcommand)]
    Flow(FlowCommand),
    /// Pack tooling (build deterministic packs, run locally)
    #[command(subcommand)]
    Pack(PackCommand),
    /// Component inspection helpers
    #[command(subcommand)]
    Component(ComponentCommand),
    /// MCP tooling (feature = "mcp")
    #[cfg(feature = "mcp")]
    #[command(subcommand)]
    Mcp(McpCommand),
}

#[derive(Subcommand)]
enum FlowCommand {
    /// Validate a flow YAML file and emit the canonical bundle JSON
    Validate(FlowValidateArgs),
}

#[derive(Args)]
struct FlowValidateArgs {
    /// Path to the flow definition (YAML)
    #[arg(short = 'f', long = "file")]
    file: PathBuf,
    /// Emit compact JSON instead of pretty-printing
    #[arg(long = "json")]
    json: bool,
}

#[derive(Subcommand)]
enum PackCommand {
    /// Build a deterministic .gtpack from a validated flow bundle
    Build(PackBuildArgs),
    /// Execute a pack locally with mocks/telemetry support
    Run(PackRunArgs),
}

#[derive(Args)]
struct PackBuildArgs {
    /// Path to the flow definition (YAML)
    #[arg(short = 'f', long = "file")]
    file: PathBuf,
    /// Output path for the generated pack
    #[arg(short = 'o', long = "out")]
    out: PathBuf,
    /// Signing mode for the generated pack
    #[arg(long = "sign", default_value = "dev", value_enum)]
    sign: PackSignArg,
    /// Optional path to pack metadata (pack.toml)
    #[arg(long = "meta")]
    meta: Option<PathBuf>,
    /// Directory containing local component builds
    #[arg(long = "component-dir", value_name = "DIR")]
    component_dir: Option<PathBuf>,
}

#[derive(Args)]
struct PackRunArgs {
    /// Path to the pack (.gtpack) to execute
    #[arg(short = 'p', long = "pack")]
    pack: PathBuf,
    /// Flow entry identifier override
    #[arg(long = "entry")]
    entry: Option<String>,
    /// JSON payload to use as run input
    #[arg(long = "input")]
    input: Option<String>,
    /// Enforcement policy for pack signatures
    #[arg(long = "policy", default_value = "devok", value_enum)]
    policy: RunPolicyArg,
    /// OTLP collector endpoint (optional)
    #[arg(long = "otlp")]
    otlp: Option<String>,
    /// Comma-separated list of allowed outbound hosts
    #[arg(long = "allow")]
    allow: Option<String>,
    /// Mocks toggle
    #[arg(long = "mocks", default_value = "on", value_enum)]
    mocks: MockSettingArg,
    /// Directory to persist run artifacts (transcripts, logs)
    #[arg(long = "artifacts")]
    artifacts: Option<PathBuf>,
}

#[derive(Subcommand)]
enum ComponentCommand {
    /// Inspect a component and print metadata
    Inspect(ComponentInspectArgs),
    /// Run diagnostics against a component
    Doctor(ComponentDoctorArgs),
}

#[derive(Args)]
struct ComponentInspectArgs {
    /// Path or identifier for the component
    target: String,
    /// Emit compact JSON instead of pretty output
    #[arg(long = "json")]
    json: bool,
}

#[derive(Args)]
struct ComponentDoctorArgs {
    /// Path or identifier for the component
    target: String,
}

#[cfg(feature = "mcp")]
#[derive(Subcommand)]
enum McpCommand {
    /// Inspect MCP provider metadata
    Doctor(McpDoctorArgs),
}

#[cfg(feature = "mcp")]
#[derive(Args)]
struct McpDoctorArgs {
    /// MCP provider identifier or config path
    provider: String,
    /// Emit compact JSON instead of pretty output
    #[arg(long = "json")]
    json: bool,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum PackSignArg {
    Dev,
    None,
}

impl From<PackSignArg> for PackSigning {
    fn from(value: PackSignArg) -> Self {
        match value {
            PackSignArg::Dev => PackSigning::Dev,
            PackSignArg::None => PackSigning::None,
        }
    }
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum RunPolicyArg {
    Strict,
    Devok,
}

impl From<RunPolicyArg> for RunPolicy {
    fn from(value: RunPolicyArg) -> Self {
        match value {
            RunPolicyArg::Strict => RunPolicy::Strict,
            RunPolicyArg::Devok => RunPolicy::DevOk,
        }
    }
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum MockSettingArg {
    On,
    Off,
}

impl From<MockSettingArg> for MockSetting {
    fn from(value: MockSettingArg) -> Self {
        match value {
            MockSettingArg::On => MockSetting::On,
            MockSettingArg::Off => MockSetting::Off,
        }
    }
}

fn split_allow_list(value: &str) -> Result<Vec<String>> {
    let hosts = value
        .split(',')
        .map(|segment| segment.trim())
        .filter(|segment| !segment.is_empty())
        .map(|segment| segment.to_string())
        .collect::<Vec<_>>();
    if hosts.is_empty() {
        anyhow::bail!("--allow expects at least one host when provided");
    }
    Ok(hosts)
}
