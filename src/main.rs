use clap::Parser;
use color_eyre::eyre;
use korrosync::cli::{Cli, Commands, resolve_db_path, run_db_command, run_user_command};
use korrosync::config::Config;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Serve => {
            let mut cfg = Config::from_env();
            if let Some(db_path) = cli.db_path {
                cfg.db.path = db_path;
            }
            korrosync::run_server(cfg).await
        }
        Commands::User(cmd) => {
            let db_path = resolve_db_path(cli.db_path);
            run_user_command(&db_path, cmd)
        }
        Commands::Db(cmd) => {
            let db_path = resolve_db_path(cli.db_path);
            run_db_command(&db_path, cmd)
        }
    }
}
