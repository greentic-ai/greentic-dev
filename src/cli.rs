use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "greentic-dev")]
#[command(version)]
#[command(about = "Greentic developer tooling CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Flow tooling (validate, lint, bundle inspection)
    #[command(subcommand)]
    Flow(FlowCommand),
    /// Pack tooling (build deterministic packs, run locally)
    #[command(subcommand)]
    Pack(PackCommand),
    /// Component tooling (scaffolding, validation, diagnostics)
    #[command(subcommand)]
    Component(ComponentCommand),
    /// Manage greentic-dev configuration
    #[command(subcommand)]
    Config(ConfigCommand),
    /// MCP tooling (feature = "mcp")
    #[cfg(feature = "mcp")]
    #[command(subcommand)]
    Mcp(McpCommand),
}

#[derive(Subcommand, Debug)]
pub enum FlowCommand {
    /// Validate a flow YAML file and emit the canonical bundle JSON
    Validate(FlowValidateArgs),
}

#[derive(Args, Debug)]
pub struct FlowValidateArgs {
    /// Path to the flow definition (YAML)
    #[arg(short = 'f', long = "file")]
    pub file: PathBuf,
    /// Emit compact JSON instead of pretty-printing
    #[arg(long = "json")]
    pub json: bool,
}

#[derive(Subcommand, Debug)]
pub enum PackCommand {
    /// Build a deterministic .gtpack from a validated flow bundle
    Build(PackBuildArgs),
    /// Execute a pack locally with mocks/telemetry support
    Run(PackRunArgs),
}

#[derive(Args, Debug)]
pub struct PackBuildArgs {
    /// Path to the flow definition (YAML)
    #[arg(short = 'f', long = "file")]
    pub file: PathBuf,
    /// Output path for the generated pack
    #[arg(short = 'o', long = "out")]
    pub out: PathBuf,
    /// Signing mode for the generated pack
    #[arg(long = "sign", default_value = "dev", value_enum)]
    pub sign: PackSignArg,
    /// Optional path to pack metadata (pack.toml)
    #[arg(long = "meta")]
    pub meta: Option<PathBuf>,
    /// Directory containing local component builds
    #[arg(long = "component-dir", value_name = "DIR")]
    pub component_dir: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct PackRunArgs {
    /// Path to the pack (.gtpack) to execute
    #[arg(short = 'p', long = "pack")]
    pub pack: PathBuf,
    /// Flow entry identifier override
    #[arg(long = "entry")]
    pub entry: Option<String>,
    /// JSON payload to use as run input
    #[arg(long = "input")]
    pub input: Option<String>,
    /// Enforcement policy for pack signatures
    #[arg(long = "policy", default_value = "devok", value_enum)]
    pub policy: RunPolicyArg,
    /// OTLP collector endpoint (optional)
    #[arg(long = "otlp")]
    pub otlp: Option<String>,
    /// Comma-separated list of allowed outbound hosts
    #[arg(long = "allow")]
    pub allow: Option<String>,
    /// Mocks toggle
    #[arg(long = "mocks", default_value = "on", value_enum)]
    pub mocks: MockSettingArg,
    /// Directory to persist run artifacts (transcripts, logs)
    #[arg(long = "artifacts")]
    pub artifacts: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum ComponentCommand {
    /// Inspect a component and print metadata
    Inspect(ComponentInspectArgs),
    /// List component templates exported by greentic-component
    Templates(ComponentTemplatesArgs),
    /// Scaffold a new component via greentic-component
    New(ComponentNewArgs),
    /// Validate a component workspace via greentic-component
    Validate(ComponentValidateArgs),
    /// Run diagnostics for a component workspace via greentic-component
    Doctor(ComponentDoctorArgs),
}

#[derive(Args, Debug, Clone)]
pub struct ComponentInspectArgs {
    /// Path or identifier for the component
    pub target: String,
    /// Emit compact JSON instead of pretty output
    #[arg(long = "json")]
    pub json: bool,
}

#[derive(Args, Debug, Clone, Default)]
pub struct ComponentTemplatesArgs {
    /// Emit JSON for the templates payload
    #[arg(long = "json")]
    pub json: bool,
    /// Enable telemetry collection by the greentic-component tool
    #[arg(long = "telemetry")]
    pub telemetry: bool,
}

#[derive(Args, Debug, Clone, Default)]
pub struct ComponentNewArgs {
    /// Component name (used for the scaffold directory)
    #[arg(long = "name", value_name = "NAME")]
    pub name: Option<String>,
    /// Target directory for the scaffold
    #[arg(long = "path", value_name = "DIR")]
    pub path: Option<PathBuf>,
    /// Template identifier from `greentic-component templates`
    #[arg(long = "template", value_name = "ID")]
    pub template: Option<String>,
    /// Reverse-DNS organization identifier (e.g., ai.greentic)
    #[arg(long = "org", value_name = "ORG")]
    pub org: Option<String>,
    /// Version for the scaffolded component (semver)
    #[arg(long = "version", value_name = "SEMVER")]
    pub version: Option<String>,
    /// License identifier (SPDX ID)
    #[arg(long = "license", value_name = "ID")]
    pub license: Option<String>,
    /// WIT world name to target
    #[arg(long = "wit-world", value_name = "WORLD")]
    pub wit_world: Option<String>,
    /// Run in non-interactive mode (no prompts)
    #[arg(long = "non-interactive")]
    pub non_interactive: bool,
    /// Skip the compile check that runs after scaffolding
    #[arg(long = "no-check")]
    pub no_check: bool,
    /// Emit JSON output for the scaffold result
    #[arg(long = "json")]
    pub json: bool,
    /// Enable telemetry collection by the greentic-component tool
    #[arg(long = "telemetry")]
    pub telemetry: bool,
}

#[derive(Args, Debug, Clone, Default)]
pub struct ComponentValidateArgs {
    /// Path to the component workspace (defaults to current directory)
    #[arg(long = "path", value_name = "DIR")]
    pub path: Option<PathBuf>,
    /// Enable telemetry collection by the greentic-component tool
    #[arg(long = "telemetry")]
    pub telemetry: bool,
}

#[derive(Args, Debug, Clone, Default)]
pub struct ComponentDoctorArgs {
    /// Path to the component workspace (defaults to current directory)
    #[arg(long = "path", value_name = "DIR")]
    pub path: Option<PathBuf>,
    /// Enable telemetry collection by the greentic-component tool
    #[arg(long = "telemetry")]
    pub telemetry: bool,
}

#[cfg(feature = "mcp")]
#[derive(Subcommand, Debug)]
pub enum McpCommand {
    /// Inspect MCP provider metadata
    Doctor(McpDoctorArgs),
}

#[cfg(feature = "mcp")]
#[derive(Args, Debug)]
pub struct McpDoctorArgs {
    /// MCP provider identifier or config path
    pub provider: String,
    /// Emit compact JSON instead of pretty output
    #[arg(long = "json")]
    pub json: bool,
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    /// Set a key in ~/.greentic/config.toml (e.g. defaults.component.org)
    Set(ConfigSetArgs),
}

#[derive(Args, Debug)]
pub struct ConfigSetArgs {
    /// Config key path (e.g. defaults.component.org)
    pub key: String,
    /// Value to assign to the key (stored as a string)
    pub value: String,
    /// Override config file path (default: ~/.greentic/config.toml)
    #[arg(long = "file")]
    pub file: Option<PathBuf>,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum PackSignArg {
    Dev,
    None,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum RunPolicyArg {
    Strict,
    Devok,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum MockSettingArg {
    On,
    Off,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parses_component_new_arguments() {
        let cli = Cli::parse_from([
            "greentic-dev",
            "component",
            "new",
            "--name",
            "demo",
            "--path",
            "./demo",
            "--template",
            "rust",
            "--org",
            "ai.greentic",
            "--version",
            "0.1.0",
            "--license",
            "Apache-2.0",
            "--wit-world",
            "component:demo",
            "--non-interactive",
            "--no-check",
            "--json",
            "--telemetry",
        ]);

        let Command::Component(ComponentCommand::New(args)) = cli.command else {
            panic!("expected component new variant");
        };
        assert_eq!(args.name.as_deref(), Some("demo"));
        assert_eq!(
            args.path.as_ref().map(|p| p.display().to_string()),
            Some("./demo".into())
        );
        assert_eq!(args.template.as_deref(), Some("rust"));
        assert_eq!(args.org.as_deref(), Some("ai.greentic"));
        assert_eq!(args.version.as_deref(), Some("0.1.0"));
        assert_eq!(args.license.as_deref(), Some("Apache-2.0"));
        assert_eq!(args.wit_world.as_deref(), Some("component:demo"));
        assert!(args.non_interactive);
        assert!(args.no_check);
        assert!(args.json);
        assert!(args.telemetry);
    }

    #[test]
    fn parses_component_templates_flag() {
        let cli = Cli::parse_from([
            "greentic-dev",
            "component",
            "templates",
            "--json",
            "--telemetry",
        ]);
        let Command::Component(ComponentCommand::Templates(args)) = cli.command else {
            panic!("expected component templates variant");
        };
        assert!(args.json);
        assert!(args.telemetry);
    }

    #[test]
    fn parses_config_set_command() {
        let cli = Cli::parse_from([
            "greentic-dev",
            "config",
            "set",
            "defaults.component.org",
            "ai.greentic",
            "--file",
            "/tmp/config.toml",
        ]);
        let Command::Config(ConfigCommand::Set(args)) = cli.command else {
            panic!("expected config set variant");
        };
        assert_eq!(args.key, "defaults.component.org");
        assert_eq!(args.value, "ai.greentic");
        assert_eq!(
            args.file.as_ref().map(|p| p.display().to_string()),
            Some("/tmp/config.toml".into())
        );
    }
}
