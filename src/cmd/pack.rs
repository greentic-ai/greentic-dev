use anyhow::Result;

use crate::cli::PackcArgs;
use crate::config;
use crate::delegate::packc::PackcDelegate;

pub fn run_new(args: &PackcArgs) -> Result<()> {
    let config = config::load()?;
    let delegate = PackcDelegate::from_config(&config)?;
    delegate.run_subcommand("new", &args.passthrough)
}
