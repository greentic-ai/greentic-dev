use clap::Parser;
use packc::cli::{self, Cli};
use tokio::runtime::Builder;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let env_filter = cli::resolve_env_filter(&cli);

    if std::env::var_os("RUST_LOG").is_none() {
        unsafe {
            std::env::set_var("RUST_LOG", &env_filter);
        }
    }

    let rt = Builder::new_multi_thread().enable_all().build()?;

    rt.block_on(async move {
        packc::telemetry::install("packc")?;
        packc::telemetry::with_task_local(async move { cli::run_with_cli(cli) }).await
    })
}
