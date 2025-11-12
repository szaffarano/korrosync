use std::error::Error;

use korrosync::config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cfg = Config::from_env();

    Ok(korrosync::run_server(cfg).await?)
}
