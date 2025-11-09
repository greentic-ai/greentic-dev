use std::ffi::OsString;

use anyhow::Result;
use greentic_dev::cli::ComponentValidateArgs;

use crate::config;
use crate::delegate::component::{ComponentDelegate, ComponentOperation};

pub fn run_validate(args: &ComponentValidateArgs) -> Result<()> {
    let config = config::load()?;
    let delegate = ComponentDelegate::from_config(&config)?;
    let cmd_args = build_args(args);
    delegate.run(ComponentOperation::Validate, cmd_args, false)?;
    Ok(())
}

fn build_args(args: &ComponentValidateArgs) -> Vec<OsString> {
    let mut result = Vec::new();
    if let Some(path) = &args.path {
        result.push(OsString::from("--path"));
        result.push(path.as_os_str().to_os_string());
    }
    if args.telemetry {
        result.push(OsString::from("--telemetry"));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::build_args;
    use greentic_dev::cli::ComponentValidateArgs;
    use std::path::PathBuf;

    #[test]
    fn includes_optional_path_flag() {
        let args = ComponentValidateArgs {
            path: Some(PathBuf::from("./path")),
            telemetry: false,
        };
        let built = build_args(&args)
            .into_iter()
            .map(|value| value.into_string().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(built, vec!["--path", "./path"]);
    }

    #[test]
    fn omits_path_when_missing() {
        let args = ComponentValidateArgs::default();
        let built = build_args(&args);
        assert!(built.is_empty());
    }

    #[test]
    fn includes_telemetry_flag_when_requested() {
        let args = ComponentValidateArgs {
            path: None,
            telemetry: true,
        };
        let built = build_args(&args)
            .into_iter()
            .map(|value| value.into_string().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(built, vec!["--telemetry"]);
    }
}
