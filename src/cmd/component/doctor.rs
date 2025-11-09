use std::ffi::OsString;

use anyhow::Result;
use greentic_dev::cli::ComponentDoctorArgs;

use crate::config;
use crate::delegate::component::{ComponentDelegate, ComponentOperation};

pub fn run_doctor(args: &ComponentDoctorArgs) -> Result<()> {
    let config = config::load()?;
    let delegate = ComponentDelegate::from_config(&config)?;
    let probe = delegate.probe()?;
    println!(
        "✓ greentic-component {} (>= {}) detected",
        probe.version,
        ComponentDelegate::minimum_version()
    );
    let cmd_args = build_args(args);
    delegate.run(ComponentOperation::Doctor, cmd_args, false)?;
    Ok(())
}

fn build_args(args: &ComponentDoctorArgs) -> Vec<OsString> {
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
    use greentic_dev::cli::ComponentDoctorArgs;
    use std::path::PathBuf;

    #[test]
    fn forwards_path_when_present() {
        let args = ComponentDoctorArgs {
            path: Some(PathBuf::from("./component")),
            telemetry: false,
        };
        let built = build_args(&args)
            .into_iter()
            .map(|value| value.into_string().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(built, vec!["--path", "./component"]);
    }

    #[test]
    fn leaves_args_empty_without_path() {
        let args = ComponentDoctorArgs::default();
        let built = build_args(&args);
        assert!(built.is_empty());
    }

    #[test]
    fn includes_telemetry_flag() {
        let args = ComponentDoctorArgs {
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
