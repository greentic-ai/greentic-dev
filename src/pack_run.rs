use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use greentic_runner::desktop::{
    HttpMock, HttpMockMode, MocksConfig, OtlpHook, Runner, SigningPolicy, ToolsMock,
};
use serde_json::{Value as JsonValue, json};

#[derive(Debug, Clone)]
pub struct PackRunConfig<'a> {
    pub pack_path: &'a Path,
    pub entry: Option<String>,
    pub input: Option<String>,
    pub policy: RunPolicy,
    pub otlp: Option<String>,
    pub allow_hosts: Option<Vec<String>>,
    pub mocks: MockSetting,
    pub artifacts_dir: Option<&'a Path>,
}

#[derive(Debug, Clone, Copy)]
pub enum RunPolicy {
    Strict,
    DevOk,
}

#[derive(Debug, Clone, Copy)]
pub enum MockSetting {
    On,
    Off,
}

pub fn run(config: PackRunConfig<'_>) -> Result<()> {
    let input_value = parse_input(config.input)?;
    let otlp_hook = config.otlp.map(|endpoint| OtlpHook {
        endpoint,
        headers: Vec::new(),
        sample_all: true,
    });
    let allow_hosts = config.allow_hosts.unwrap_or_default();
    let mocks_config = build_mocks_config(config.mocks, allow_hosts)?;

    let artifacts_override = config.artifacts_dir.map(|dir| dir.to_path_buf());
    if let Some(dir) = &artifacts_override {
        fs::create_dir_all(dir)
            .with_context(|| format!("failed to create artifacts directory {}", dir.display()))?;
    }

    let runner = Runner::new();
    let run_result = runner
        .run_pack_with(config.pack_path, |opts| {
            opts.entry_flow = config.entry.clone();
            opts.input = input_value.clone();
            opts.signing = signing_policy(config.policy);
            if let Some(hook) = otlp_hook.clone() {
                opts.otlp = Some(hook);
            }
            opts.mocks = mocks_config.clone();
            opts.artifacts_dir = artifacts_override.clone();
        })
        .context("pack execution failed")?;

    let rendered =
        serde_json::to_string_pretty(&run_result).context("failed to render run result JSON")?;
    println!("{rendered}");

    Ok(())
}

fn parse_input(input: Option<String>) -> Result<JsonValue> {
    if let Some(raw) = input {
        if raw.trim().is_empty() {
            return Ok(json!({}));
        }
        serde_json::from_str(&raw).context("failed to parse --input JSON")
    } else {
        Ok(json!({}))
    }
}

fn build_mocks_config(setting: MockSetting, allow_hosts: Vec<String>) -> Result<MocksConfig> {
    let mut config = MocksConfig {
        net_allowlist: allow_hosts
            .into_iter()
            .map(|host| host.trim().to_ascii_lowercase())
            .filter(|host| !host.is_empty())
            .collect(),
        ..MocksConfig::default()
    };

    if matches!(setting, MockSetting::On) {
        config.http = Some(HttpMock {
            record_replay_dir: None,
            mode: HttpMockMode::RecordReplay,
            rewrites: Vec::new(),
        });

        let tools_dir = PathBuf::from(".greentic").join("mocks").join("tools");
        fs::create_dir_all(&tools_dir)
            .with_context(|| format!("failed to create {}", tools_dir.display()))?;
        config.mcp_tools = Some(ToolsMock {
            directory: None,
            script_dir: Some(tools_dir),
            short_circuit: true,
        });
    }

    Ok(config)
}

fn signing_policy(policy: RunPolicy) -> SigningPolicy {
    match policy {
        RunPolicy::Strict => SigningPolicy::Strict,
        RunPolicy::DevOk => SigningPolicy::DevOk,
    }
}
