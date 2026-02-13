use std::fs;

use clap::Parser;
use color_eyre::eyre::{self, Context};
use korrosync::cli::{Cli, Commands, DbCommands, UserCommands};
use korrosync::config::Config;
use korrosync::model::User;
use korrosync::service::db::{KorrosyncService, KorrosyncServiceRedb};

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
            let service = KorrosyncServiceRedb::new(&db_path).context("Failed to open database")?;

            match cmd {
                UserCommands::Create { username, password } => {
                    let password = resolve_password(password)?;
                    let user = User::new(&username, &password)
                        .map_err(|e| eyre::eyre!("Failed to create user: {}", e))?;
                    service
                        .create_or_update_user(user)
                        .context("Failed to save user")?;
                    println!("User '{}' created successfully", username);
                }
                UserCommands::List => {
                    let users = service.list_users().context("Failed to list users")?;
                    if users.is_empty() {
                        println!("No users found");
                    } else {
                        println!("{:<20} {}", "USERNAME", "LAST ACTIVITY");
                        println!("{}", "-".repeat(40));
                        for user in &users {
                            let activity = user
                                .last_activity()
                                .map(|ts| {
                                    chrono::DateTime::from_timestamp_millis(ts)
                                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                                        .unwrap_or_else(|| ts.to_string())
                                })
                                .unwrap_or_else(|| "never".to_string());
                            println!("{:<20} {}", user.username(), activity);
                        }
                        println!("\nTotal: {} user(s)", users.len());
                    }
                }
                UserCommands::Remove { username } => {
                    let deleted = service
                        .delete_user(username.clone())
                        .context("Failed to delete user")?;
                    if deleted {
                        println!("User '{}' removed successfully", username);
                    } else {
                        println!("User '{}' not found", username);
                    }
                }
                UserCommands::ResetPassword { username, password } => {
                    let password = resolve_password(password)?;
                    let existing = service
                        .get_user(username.clone())
                        .context("Failed to query user")?;
                    if existing.is_none() {
                        eyre::bail!("User '{}' not found", username);
                    }
                    let user = User::new(&username, &password)
                        .map_err(|e| eyre::eyre!("Failed to hash password: {}", e))?;
                    service
                        .create_or_update_user(user)
                        .context("Failed to update user")?;
                    println!("Password for user '{}' reset successfully", username);
                }
            }
            Ok(())
        }
        Commands::Db(cmd) => {
            let db_path = resolve_db_path(cli.db_path);

            match cmd {
                DbCommands::Info => {
                    let metadata = fs::metadata(&db_path);
                    println!("Database path: {}", db_path);
                    match metadata {
                        Ok(meta) => {
                            println!("Database size: {} bytes", meta.len());
                        }
                        Err(_) => {
                            println!("Database file does not exist yet");
                        }
                    }
                    if let Ok(service) = KorrosyncServiceRedb::new(&db_path) {
                        let users = service.list_users().unwrap_or_default();
                        println!("Users: {}", users.len());
                    }
                }
                DbCommands::Backup { output } => {
                    fs::copy(&db_path, &output).context("Failed to backup database")?;
                    println!("Database backed up to '{}'", output);
                }
            }
            Ok(())
        }
    }
}

fn resolve_db_path(cli_override: Option<String>) -> String {
    cli_override.unwrap_or_else(|| Config::from_env().db.path)
}

fn resolve_password(password: String) -> eyre::Result<String> {
    if password != "-" {
        return Ok(password);
    }
    let mut buf = String::new();
    std::io::stdin()
        .read_line(&mut buf)
        .context("Failed to read password from stdin")?;
    let password = buf.trim_end_matches('\n').trim_end_matches('\r');
    if password.is_empty() {
        eyre::bail!("Password cannot be empty");
    }
    Ok(password.to_string())
}
