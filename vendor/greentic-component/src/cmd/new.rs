#![cfg(feature = "cli")]

use std::env;
use std::io::{Write, stdout};
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::time::Instant;

use anyhow::{Context, Result};
use clap::Args;
use serde::Serialize;
use serde_json::json;

use crate::cmd::post::{self, GitInitStatus, PostInitReport};
use crate::scaffold::deps::DependencyMode;
use crate::scaffold::engine::{
    DEFAULT_WIT_WORLD, ScaffoldEngine, ScaffoldOutcome, ScaffoldRequest,
};
use crate::scaffold::validate::{self, ComponentName, OrgNamespace, ValidationError};

type ValidationResult<T> = std::result::Result<T, ValidationError>;
const SKIP_GIT_ENV: &str = "GREENTIC_SKIP_GIT";

#[derive(Args, Debug, Clone)]
pub struct NewArgs {
    /// Name for the component (kebab-or-snake case)
    #[arg(long = "name", value_name = "kebab_or_snake", required = true)]
    pub name: String,
    /// Path to create the component (defaults to ./<name>)
    #[arg(long = "path", value_name = "dir")]
    pub path: Option<PathBuf>,
    /// Template identifier to scaffold from
    #[arg(
        long = "template",
        default_value = "rust-wasi-p2-min",
        value_name = "id"
    )]
    pub template: String,
    /// Reverse DNS-style organisation identifier
    #[arg(
        long = "org",
        default_value = "ai.greentic",
        value_name = "reverse.dns"
    )]
    pub org: String,
    /// Initial component version
    #[arg(long = "version", default_value = "0.1.0", value_name = "semver")]
    pub version: String,
    /// License to embed into generated sources
    #[arg(long = "license", default_value = "MIT", value_name = "id")]
    pub license: String,
    /// Exported WIT world name
    #[arg(
        long = "wit-world",
        default_value = DEFAULT_WIT_WORLD,
        value_name = "name"
    )]
    pub wit_world: String,
    /// Run without prompting for confirmation
    #[arg(long = "non-interactive")]
    pub non_interactive: bool,
    /// Skip the post-scaffold cargo check (hidden flag for testing/local dev)
    #[arg(long = "no-check", hide = true)]
    pub no_check: bool,
    /// Skip git initialization after scaffolding
    #[arg(long = "no-git")]
    pub no_git: bool,
    /// Emit JSON instead of human-readable output
    #[arg(long = "json")]
    pub json: bool,
}

pub fn run(args: NewArgs, engine: &ScaffoldEngine) -> Result<()> {
    let request = match build_request(&args) {
        Ok(req) => req,
        Err(err) => {
            emit_validation_failure(&err, args.json)?;
            return Err(err.into());
        }
    };
    if !args.json {
        println!("processing...");
        println!(
            "  - template: {} -> {}",
            request.template_id,
            request.path.display()
        );
        stdout().flush().ok();
    }
    let scaffold_started = Instant::now();
    let outcome = engine.scaffold(request)?;
    if !args.json {
        println!("scaffolded files in {:.2?}", scaffold_started.elapsed());
        stdout().flush().ok();
    }
    let post_started = Instant::now();
    let skip_git = should_skip_git(&args);
    let post_init = post::run_post_init(&outcome, skip_git);
    let compile_check = run_compile_check(&outcome.path, args.no_check)?;
    if args.json {
        let payload = NewCliOutput {
            scaffold: &outcome,
            compile_check: &compile_check,
            post_init: &post_init,
        };
        print_json(&payload)?;
    } else {
        print_human(&outcome, &compile_check, &post_init);
        println!("post-init + checks in {:.2?}", post_started.elapsed());
    }
    if compile_check.ran && !compile_check.passed {
        anyhow::bail!("cargo check --target wasm32-wasip2 failed");
    }
    Ok(())
}

fn build_request(args: &NewArgs) -> ValidationResult<ScaffoldRequest> {
    let component_name = ComponentName::parse(&args.name)?;
    let org = OrgNamespace::parse(&args.org)?;
    let version = validate::normalize_version(&args.version)?;
    let target_path = resolve_path(&component_name, args.path.as_deref())?;
    Ok(ScaffoldRequest {
        name: component_name.into_string(),
        path: target_path,
        template_id: args.template.clone(),
        org: org.into_string(),
        version,
        license: args.license.clone(),
        wit_world: args.wit_world.clone(),
        non_interactive: args.non_interactive,
        year_override: None,
        dependency_mode: DependencyMode::from_env(),
    })
}

fn resolve_path(name: &ComponentName, provided: Option<&Path>) -> ValidationResult<PathBuf> {
    let path = validate::resolve_target_path(name, provided)?;
    Ok(path)
}

fn print_json<T: Serialize>(value: &T) -> Result<()> {
    let mut handle = std::io::stdout();
    serde_json::to_writer_pretty(&mut handle, value)?;
    handle.write_all(b"\n").ok();
    Ok(())
}

fn print_human(outcome: &ScaffoldOutcome, check: &CompileCheckReport, post: &PostInitReport) {
    println!("{}", outcome.human_summary());
    print_template_metadata(outcome);
    for path in &outcome.created {
        println!("  - {path}");
    }
    print_git_summary(&post.git);
    if !check.ran {
        println!("cargo check (wasm32-wasip2): skipped (--no-check)");
    } else if check.passed {
        println!("cargo check (wasm32-wasip2): ok");
    } else {
        println!(
            "cargo check (wasm32-wasip2): FAILED (exit code {:?})",
            check.exit_code
        );
        if let Some(stderr) = &check.stderr
            && !stderr.is_empty()
        {
            println!("{stderr}");
        }
    }
    if !post.next_steps.is_empty() {
        println!("Next steps:");
        for step in &post.next_steps {
            println!("  $ {step}");
        }
    }
}

fn print_git_summary(report: &post::GitInitReport) {
    match report.status {
        GitInitStatus::Initialized => {
            if let Some(commit) = &report.commit {
                println!("git init: ok (commit {commit})");
            } else {
                println!("git init: ok");
            }
        }
        GitInitStatus::AlreadyPresent => {
            println!(
                "git init: skipped ({})",
                report
                    .message
                    .as_deref()
                    .unwrap_or("directory already contains .git")
            );
        }
        GitInitStatus::InsideWorktree => {
            println!(
                "git init: skipped ({})",
                report
                    .message
                    .as_deref()
                    .unwrap_or("already inside an existing git worktree")
            );
        }
        GitInitStatus::Skipped => {
            println!(
                "git init: skipped ({})",
                report.message.as_deref().unwrap_or("not requested")
            );
        }
        GitInitStatus::Failed => {
            println!(
                "git init: failed ({})",
                report
                    .message
                    .as_deref()
                    .unwrap_or("see logs for more details")
            );
        }
    }
}

fn print_template_metadata(outcome: &ScaffoldOutcome) {
    match &outcome.template_description {
        Some(desc) => println!("Template: {} — {desc}", outcome.template),
        None => println!("Template: {}", outcome.template),
    }
    if !outcome.template_tags.is_empty() {
        println!("  tags: {}", outcome.template_tags.join(", "));
    }
}

fn should_skip_git(args: &NewArgs) -> bool {
    if args.no_git {
        return true;
    }
    match env::var(SKIP_GIT_ENV) {
        Ok(value) => matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes"
        ),
        Err(_) => false,
    }
}

fn run_compile_check(path: &Path, skip: bool) -> Result<CompileCheckReport> {
    const COMMAND_DISPLAY: &str = "cargo check --target wasm32-wasip2";
    if skip {
        return Ok(CompileCheckReport::skipped(COMMAND_DISPLAY));
    }
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut cmd = Command::new(cargo);
    cmd.arg("check").arg("--target").arg("wasm32-wasip2");
    cmd.current_dir(path);
    let start = Instant::now();
    let output = cmd
        .output()
        .with_context(|| format!("failed to run `{COMMAND_DISPLAY}`"))?;
    let duration_ms = start.elapsed().as_millis();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Ok(CompileCheckReport {
        command: COMMAND_DISPLAY.to_string(),
        ran: true,
        passed: output.status.success(),
        exit_code: output.status.code(),
        duration_ms: Some(duration_ms),
        stdout: if stdout.is_empty() {
            None
        } else {
            Some(stdout)
        },
        stderr: if stderr.is_empty() {
            None
        } else {
            Some(stderr)
        },
        reason: None,
    })
}

fn emit_validation_failure(err: &ValidationError, json: bool) -> Result<()> {
    if json {
        let payload = json!({
            "error": {
                "kind": "validation",
                "code": err.code(),
                "message": err.to_string()
            }
        });
        print_json(&payload)?;
        process::exit(1);
    }
    Ok(())
}

#[derive(Serialize)]
struct NewCliOutput<'a> {
    scaffold: &'a ScaffoldOutcome,
    compile_check: &'a CompileCheckReport,
    post_init: &'a PostInitReport,
}

#[derive(Debug, Serialize)]
struct CompileCheckReport {
    command: String,
    ran: bool,
    passed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_ms: Option<u128>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stdout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stderr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

impl CompileCheckReport {
    fn skipped(command: &str) -> Self {
        Self {
            command: command.to_string(),
            ran: false,
            passed: true,
            exit_code: None,
            duration_ms: None,
            stdout: None,
            stderr: None,
            reason: Some("skipped (--no-check)".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_path_uses_name() {
        let args = NewArgs {
            name: "demo-component".into(),
            path: None,
            template: "rust-wasi-p2-min".into(),
            org: "ai.greentic".into(),
            version: "0.1.0".into(),
            license: "MIT".into(),
            wit_world: DEFAULT_WIT_WORLD.into(),
            non_interactive: false,
            no_check: false,
            no_git: false,
            json: false,
        };
        let request = build_request(&args).unwrap();
        assert!(request.path.ends_with("demo-component"));
    }
}
