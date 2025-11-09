use std::ffi::OsString;
use std::path::Path;

use anyhow::{Context, Result};
use greentic_dev::cli::ComponentNewArgs;

use crate::cmd::component::{ComponentNewResponse, emit_human_hint, emit_json_error};
use crate::config::{self, ComponentDefaults};
use crate::delegate::component::{ComponentDelegate, ComponentOperation};

const ERROR_CODE_NEW: &str = "E_COMPONENT_SCAFFOLD";

pub fn run_new(args: &ComponentNewArgs) -> Result<()> {
    if args.json {
        return run_new_json(args);
    }
    run_new_human(args)
}

fn run_new_json(args: &ComponentNewArgs) -> Result<()> {
    match fetch_new_payload(args) {
        Ok(payload) => {
            let wrapper = payload.into_wrapper();
            println!("{}", serde_json::to_string(&wrapper)?);
            Ok(())
        }
        Err(error) => {
            emit_json_error("new", ERROR_CODE_NEW, &error.to_string())?;
            Err(error)
        }
    }
}

fn fetch_new_payload(args: &ComponentNewArgs) -> Result<ComponentNewResponse> {
    let config = config::load()?;
    let delegate = ComponentDelegate::from_config(&config)?;
    let cmd_args = build_delegate_args(args, &config.defaults.component);
    let output = delegate.run(ComponentOperation::New, cmd_args, true)?;
    let value = output
        .json
        .context("component new command did not emit JSON")?;
    let payload: ComponentNewResponse =
        serde_json::from_value(value).context("failed to parse component new JSON payload")?;
    Ok(payload)
}

fn run_new_human(args: &ComponentNewArgs) -> Result<()> {
    let config = config::load()?;
    let delegate = ComponentDelegate::from_config(&config)?;
    let cmd_args = build_delegate_args(args, &config.defaults.component);
    if let Err(error) = delegate.run(ComponentOperation::New, cmd_args, false) {
        emit_human_hint("Component scaffolding failed", &error);
        return Err(error);
    }
    Ok(())
}

fn build_delegate_args(args: &ComponentNewArgs, defaults: &ComponentDefaults) -> Vec<OsString> {
    let mut result = Vec::new();
    push_opt_str(&mut result, "--name", args.name.as_deref());
    push_opt_path(&mut result, "--path", args.path.as_deref());
    let template = args.template.as_deref().or(defaults.template.as_deref());
    push_opt_str(&mut result, "--template", template);
    let org = args.org.as_deref().or(defaults.org.as_deref());
    push_opt_str(&mut result, "--org", org);
    push_opt_str(&mut result, "--version", args.version.as_deref());
    push_opt_str(&mut result, "--license", args.license.as_deref());
    push_opt_str(&mut result, "--wit-world", args.wit_world.as_deref());
    if args.non_interactive {
        result.push(OsString::from("--non-interactive"));
    }
    if args.no_check {
        result.push(OsString::from("--no-check"));
    }
    if args.json {
        result.push(OsString::from("--json"));
    }
    if args.telemetry {
        result.push(OsString::from("--telemetry"));
    }
    result
}

fn push_opt_str(args: &mut Vec<OsString>, flag: &str, value: Option<&str>) {
    if let Some(value) = value {
        args.push(OsString::from(flag));
        args.push(OsString::from(value));
    }
}

fn push_opt_path(args: &mut Vec<OsString>, flag: &str, value: Option<&Path>) {
    if let Some(value) = value {
        args.push(OsString::from(flag));
        args.push(value.as_os_str().to_os_string());
    }
}

#[cfg(test)]
mod tests {
    use super::build_delegate_args;
    use crate::config::ComponentDefaults;
    use greentic_dev::cli::ComponentNewArgs;
    use std::path::PathBuf;

    #[test]
    fn builds_expected_argument_sequence() {
        let args = ComponentNewArgs {
            name: Some("demo".into()),
            path: Some(PathBuf::from("./demo")),
            template: Some("rust-wasi".into()),
            org: Some("ai.greentic".into()),
            version: Some("0.1.0".into()),
            license: Some("Apache-2.0".into()),
            wit_world: Some("component:demo".into()),
            non_interactive: true,
            no_check: true,
            json: true,
            telemetry: true,
        };
        let defaults = ComponentDefaults::default();

        let built = build_delegate_args(&args, &defaults)
            .into_iter()
            .map(|value| value.into_string().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            built,
            vec![
                "--name",
                "demo",
                "--path",
                "./demo",
                "--template",
                "rust-wasi",
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
                "--telemetry"
            ]
        );
    }

    #[test]
    fn skips_absent_optional_arguments() {
        let args = ComponentNewArgs::default();
        let defaults = ComponentDefaults::default();
        let built = build_delegate_args(&args, &defaults);
        assert!(built.is_empty());
    }

    #[test]
    fn applies_component_defaults_for_org_and_template() {
        let args = ComponentNewArgs::default();
        let defaults = ComponentDefaults {
            org: Some("ai.greentic".into()),
            template: Some("rust-wasi-p2-min".into()),
        };
        let built = build_delegate_args(&args, &defaults)
            .into_iter()
            .map(|value| value.into_string().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            built,
            vec!["--template", "rust-wasi-p2-min", "--org", "ai.greentic"]
        );
    }
}
