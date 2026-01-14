use anyhow::{Context, Result};
use clap::Parser;
use std::ffi::OsString;
use std::path::PathBuf;

use greentic_dev::cli::McpCommand;
use greentic_dev::cli::{Cli, Command, MockSettingArg, PackRunArgs, RunPolicyArg};
use greentic_dev::pack_run::{MockSetting, PackRunConfig, RunPolicy};
use greentic_dev::passthrough::{resolve_binary, run_passthrough};

use greentic_dev::cbor_cmd;
use greentic_dev::cmd::config;
use greentic_dev::mcp_cmd;
use greentic_dev::pack_run;
use greentic_dev::secrets_cli::run_secrets_command;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Flow(args) => {
            let bin = resolve_binary("greentic-flow")?;
            let status = run_passthrough(&bin, &args.args, false)?;
            std::process::exit(status.code().unwrap_or(1));
        }
        Command::Pack(args) => {
            let subcommand = args.args.first().and_then(|s| s.to_str());
            if subcommand == Some("run") {
                let wants_help = args.args.iter().any(|arg| arg == "-h" || arg == "--help");
                if wants_help {
                    let bin = resolve_binary("greentic-runner-cli")?;
                    let run_args: Vec<OsString> = args.args.iter().skip(1).cloned().collect();
                    let status = run_passthrough(&bin, &run_args, false)?;
                    std::process::exit(status.code().unwrap_or(1));
                }
                run_pack_run_wrapper(&args.args)?;
                return Ok(());
            }

            let bin = resolve_binary("greentic-pack")?;
            let status = run_passthrough(&bin, &args.args, false)?;
            std::process::exit(status.code().unwrap_or(1));
        }
        Command::Component(args) => {
            let bin = resolve_binary("greentic-component")?;
            let status = run_passthrough(&bin, &args.args, false)?;
            std::process::exit(status.code().unwrap_or(1));
        }
        Command::Config(config_cmd) => config::run(config_cmd),
        Command::Cbor(args) => cbor_cmd::run(args),
        Command::Mcp(mcp) => match mcp {
            McpCommand::Doctor(args) => mcp_cmd::doctor(&args.provider, args.json),
        },
        Command::Gui(args) => {
            let bin = resolve_binary("greentic-gui")?;
            let status = run_passthrough(&bin, &args.args, false)?;
            std::process::exit(status.code().unwrap_or(1));
        }
        Command::Secrets(secrets) => run_secrets_command(secrets),
    }
}

fn run_pack_run_wrapper(args: &[OsString]) -> Result<()> {
    let mut parse_args: Vec<OsString> = vec!["greentic-dev-pack-run".into()];
    parse_args.extend(args.iter().skip(1).cloned());
    let run_args = PackRunWrapper::try_parse_from(parse_args)?.args;

    let mock_external_payload = if let Some(payload_path) = &run_args.mock_external_payload {
        let data = std::fs::read_to_string(payload_path).with_context(|| {
            format!(
                "failed to read mock external payload {}",
                payload_path.display()
            )
        })?;
        Some(serde_json::from_str(&data).context("invalid mock external payload JSON")?)
    } else {
        None
    };

    let allow_hosts = run_args
        .allow
        .as_deref()
        .map(|raw| raw.split(',').map(|item| item.trim().to_string()).collect());

    let pack_path: PathBuf = run_args.pack.clone();
    let artifacts_dir = run_args.artifacts.clone();
    let secrets_seed = run_args.secrets_seed.clone();

    let config = PackRunConfig {
        pack_path: &pack_path,
        entry: run_args.entry,
        input: run_args.input,
        policy: match run_args.policy {
            RunPolicyArg::Strict => RunPolicy::Strict,
            RunPolicyArg::Devok => RunPolicy::DevOk,
        },
        otlp: run_args.otlp,
        allow_hosts,
        mocks: match run_args.mocks {
            MockSettingArg::On => MockSetting::On,
            MockSettingArg::Off => MockSetting::Off,
        },
        artifacts_dir: artifacts_dir.as_deref(),
        json: run_args.json,
        offline: run_args.offline,
        mock_exec: run_args.mock_exec,
        allow_external: run_args.allow_external,
        mock_external: run_args.mock_external,
        mock_external_payload,
        secrets_seed: secrets_seed.as_deref(),
    };

    pack_run::run(config)
}

#[derive(Parser, Debug)]
struct PackRunWrapper {
    #[command(flatten)]
    args: PackRunArgs,
}
