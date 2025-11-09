use std::ffi::OsString;

use anyhow::{Context, Result};
use greentic_dev::cli::ComponentTemplatesArgs;

use crate::cmd::component::{
    ComponentTemplatesResponse, TOOL_NAME, emit_human_hint, emit_json_error,
};
use crate::config;
use crate::delegate::component::{ComponentDelegate, ComponentOperation};

const ERROR_CODE_TEMPLATES: &str = "E_COMPONENT_TEMPLATES";

pub fn run_templates(args: &ComponentTemplatesArgs) -> Result<()> {
    if args.json {
        return run_templates_json(args);
    }
    run_templates_human(args)
}

fn run_templates_json(args: &ComponentTemplatesArgs) -> Result<()> {
    match fetch_templates_payload(args) {
        Ok(payload) => {
            print_templates_wrapper(payload)?;
            Ok(())
        }
        Err(error) => {
            emit_json_error("templates", ERROR_CODE_TEMPLATES, &error.to_string())?;
            Err(error)
        }
    }
}

fn run_templates_human(args: &ComponentTemplatesArgs) -> Result<()> {
    let config = config::load()?;
    let delegate = ComponentDelegate::from_config(&config)?;
    let delegate_args = build_template_args(args);
    if let Err(error) = delegate.run(ComponentOperation::Templates, delegate_args, false) {
        emit_human_hint("Component template listing failed", &error);
        return Err(error);
    }
    Ok(())
}

fn fetch_templates_payload(args: &ComponentTemplatesArgs) -> Result<ComponentTemplatesResponse> {
    let config = config::load()?;
    let delegate = ComponentDelegate::from_config(&config)?;
    let delegate_args = build_template_args(args);
    let output = delegate.run(ComponentOperation::Templates, delegate_args, true)?;
    let value = output
        .json
        .context("component templates command did not emit JSON")?;
    let payload: ComponentTemplatesResponse = serde_json::from_value(value)
        .context("failed to parse component templates JSON payload")?;
    Ok(payload)
}

fn build_template_args(args: &ComponentTemplatesArgs) -> Vec<OsString> {
    let mut result = Vec::new();
    if args.telemetry {
        result.push(OsString::from("--telemetry"));
    }
    result
}

fn print_templates_wrapper(payload: ComponentTemplatesResponse) -> Result<()> {
    let ComponentTemplatesResponse { templates, extra } = payload;
    let mut wrapper = serde_json::json!({
        "tool": TOOL_NAME,
        "command": "templates",
        "ok": true,
        "templates": templates,
    });
    if !extra.is_empty() {
        wrapper
            .as_object_mut()
            .expect("wrapper is object")
            .insert("payload".to_string(), serde_json::Value::Object(extra));
    }
    println!("{}", serde_json::to_string(&wrapper)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::build_template_args;
    use greentic_dev::cli::ComponentTemplatesArgs;

    #[test]
    fn telemetry_flag_is_forwarded() {
        let args = ComponentTemplatesArgs {
            json: false,
            telemetry: true,
        };
        let built = build_template_args(&args)
            .into_iter()
            .map(|arg| arg.into_string().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(built, vec!["--telemetry"]);
    }

    #[test]
    fn empty_when_disabled() {
        let args = ComponentTemplatesArgs::default();
        let built = build_template_args(&args);
        assert!(built.is_empty());
    }
}
