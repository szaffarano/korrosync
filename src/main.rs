use color_eyre::eyre;
use korrosync::config::Config;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let cfg = Config::from_env();

    korrosync::run_server(cfg).await
}
