use std::fs;
use std::io::BufRead;

use clap::{Parser, Subcommand};
use color_eyre::eyre::{self, Context};

use crate::config::Config;
use crate::model::User;
use crate::service::db::{KorrosyncService, KorrosyncServiceRedb};

#[derive(Parser)]
#[command(name = "korrosync", version, about = "KOReader synchronization server")]
pub struct Cli {
    /// Path to the database file (overrides KORROSYNC_DB_PATH env var)
    #[arg(long, global = true)]
    pub db_path: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the sync server
    Serve,
    /// User management commands
    #[command(subcommand)]
    User(UserCommands),
    /// Database maintenance commands
    #[command(subcommand)]
    Db(DbCommands),
}

#[derive(Subcommand, Debug)]
pub enum UserCommands {
    /// Create a new user
    Create {
        #[arg(short, long)]
        username: String,
        /// Password (use '-' to read from stdin)
        #[arg(short, long)]
        password: String,
    },
    /// List all users
    List,
    /// Remove a user
    Remove {
        #[arg(short, long)]
        username: String,
    },
    /// Reset a user's password
    ResetPassword {
        #[arg(short, long)]
        username: String,
        /// Password (use '-' to read from stdin)
        #[arg(short, long)]
        password: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum DbCommands {
    /// Show database path and basic stats
    Info,
    /// Backup the database to a file
    Backup {
        /// Output file path
        #[arg(short, long)]
        output: String,
    },
}

/// Resolve the database path from a CLI override or environment/defaults.
pub fn resolve_db_path(cli_override: Option<String>) -> String {
    cli_override.unwrap_or_else(|| Config::from_env().db.path)
}

/// Resolve a password argument, reading a single line from `reader` when the value is `-`.
pub fn resolve_password_from_reader(
    password: String,
    mut reader: impl BufRead,
) -> eyre::Result<String> {
    if password != "-" {
        return Ok(password);
    }
    let mut buf = String::new();
    reader
        .read_line(&mut buf)
        .context("Failed to read password from stdin")?;
    let password = buf.trim_end_matches('\n').trim_end_matches('\r');
    if password.is_empty() {
        eyre::bail!("Password cannot be empty");
    }
    Ok(password.to_string())
}

/// Resolve a password argument, reading from stdin when the value is `-`.
pub fn resolve_password(password: String) -> eyre::Result<String> {
    resolve_password_from_reader(password, std::io::stdin().lock())
}

/// Execute a user-management subcommand against the database at `db_path`.
pub fn run_user_command(db_path: &str, cmd: UserCommands) -> eyre::Result<()> {
    let service = KorrosyncServiceRedb::new(db_path).context("Failed to open database")?;

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
                println!("{:<20} LAST ACTIVITY", "USERNAME");
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

/// Execute a database maintenance subcommand against the database at `db_path`.
pub fn run_db_command(db_path: &str, cmd: DbCommands) -> eyre::Result<()> {
    match cmd {
        DbCommands::Info => {
            let metadata = fs::metadata(db_path);
            println!("Database path: {}", db_path);
            match metadata {
                Ok(meta) => {
                    println!("Database size: {} bytes", meta.len());
                }
                Err(_) => {
                    println!("Database file does not exist yet");
                }
            }
            if let Ok(service) = KorrosyncServiceRedb::new(db_path) {
                let users = service.list_users().unwrap_or_default();
                println!("Users: {}", users.len());
            }
        }
        DbCommands::Backup { output } => {
            fs::copy(db_path, &output).context("Failed to backup database")?;
            println!("Database backed up to '{}'", output);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::User;
    use crate::service::db::{KorrosyncService, KorrosyncServiceRedb};
    use std::io::Cursor;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn resolve_db_path_uses_override() {
        assert_eq!(
            resolve_db_path(Some("/tmp/custom.redb".into())),
            "/tmp/custom.redb"
        );
    }

    #[test]
    fn resolve_db_path_falls_back_to_env() {
        temp_env::with_vars([("KORROSYNC_DB_PATH", Some("/tmp/from-env.redb"))], || {
            assert_eq!(resolve_db_path(None), "/tmp/from-env.redb");
        });
    }

    #[test]
    fn resolve_password_returns_literal() {
        let password =
            resolve_password_from_reader("secret".into(), Cursor::new("")).expect("password");
        assert_eq!(password, "secret");
    }

    #[test]
    fn resolve_password_reads_from_reader() {
        let password = resolve_password_from_reader("-".into(), Cursor::new("from-stdin\n"))
            .expect("password");
        assert_eq!(password, "from-stdin");
    }

    #[test]
    fn resolve_password_trims_crlf() {
        let password =
            resolve_password_from_reader("-".into(), Cursor::new("pw\r\n")).expect("password");
        assert_eq!(password, "pw");
    }

    #[test]
    fn resolve_password_rejects_empty_stdin() {
        let err = resolve_password_from_reader("-".into(), Cursor::new("\n")).unwrap_err();
        assert!(err.to_string().contains("Password cannot be empty"));
    }

    #[test]
    fn user_create_list_remove_and_reset_password() {
        let db = NamedTempFile::new().unwrap();
        let db_path = db.path().to_string_lossy().to_string();

        run_user_command(
            &db_path,
            UserCommands::Create {
                username: "alice".into(),
                password: "secret".into(),
            },
        )
        .expect("create user");

        run_user_command(&db_path, UserCommands::List).expect("list users");

        run_user_command(
            &db_path,
            UserCommands::ResetPassword {
                username: "alice".into(),
                password: "new-secret".into(),
            },
        )
        .expect("reset password");

        {
            let service = KorrosyncServiceRedb::new(&db_path).unwrap();
            let user = service.get_user("alice".into()).unwrap().unwrap();
            assert!(user.check("new-secret").unwrap());
        }

        run_user_command(
            &db_path,
            UserCommands::Remove {
                username: "alice".into(),
            },
        )
        .expect("remove user");

        let service = KorrosyncServiceRedb::new(&db_path).unwrap();
        assert!(service.get_user("alice".into()).unwrap().is_none());
    }

    #[test]
    fn user_list_empty_and_remove_missing() {
        let db = NamedTempFile::new().unwrap();
        let db_path = db.path().to_string_lossy().to_string();

        run_user_command(&db_path, UserCommands::List).expect("list empty");
        run_user_command(
            &db_path,
            UserCommands::Remove {
                username: "nobody".into(),
            },
        )
        .expect("remove missing");
    }

    #[test]
    fn user_list_formats_last_activity() {
        let db = NamedTempFile::new().unwrap();
        let db_path = db.path().to_string_lossy().to_string();
        {
            let service = KorrosyncServiceRedb::new(&db_path).unwrap();

            let mut with_activity = User::new("active", "pw").unwrap();
            with_activity.set_last_activity(1_609_459_200_000);
            service.create_or_update_user(with_activity).unwrap();

            let mut invalid_ts = User::new("weird", "pw").unwrap();
            invalid_ts.set_last_activity(i64::MAX);
            service.create_or_update_user(invalid_ts).unwrap();

            let never = User::new("never", "pw").unwrap();
            service.create_or_update_user(never).unwrap();
        }

        run_user_command(&db_path, UserCommands::List).expect("list users");
    }

    #[test]
    fn reset_password_missing_user_errors() {
        let db = NamedTempFile::new().unwrap();
        let db_path = db.path().to_string_lossy().to_string();

        let err = run_user_command(
            &db_path,
            UserCommands::ResetPassword {
                username: "missing".into(),
                password: "pw".into(),
            },
        )
        .unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn db_info_and_backup() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("db.redb");
        let backup_path = dir.path().join("backup.redb");

        // Info before the DB exists
        run_db_command(db_path.to_str().unwrap(), DbCommands::Info).expect("info missing db");

        let service = KorrosyncServiceRedb::new(&db_path).unwrap();
        service
            .create_or_update_user(User::new("bob", "pw").unwrap())
            .unwrap();

        run_db_command(db_path.to_str().unwrap(), DbCommands::Info).expect("info existing db");

        run_db_command(
            db_path.to_str().unwrap(),
            DbCommands::Backup {
                output: backup_path.to_string_lossy().to_string(),
            },
        )
        .expect("backup");

        assert!(backup_path.exists());
    }

    #[test]
    fn db_backup_missing_source_errors() {
        let err = run_db_command(
            "/tmp/korrosync-does-not-exist-db.redb",
            DbCommands::Backup {
                output: "/tmp/korrosync-backup-out.redb".into(),
            },
        )
        .unwrap_err();
        assert!(err.to_string().contains("Failed to backup database"));
    }

    #[test]
    fn run_user_command_invalid_db_path_errors() {
        let err =
            run_user_command("/dev/null/not-a-valid-db-path", UserCommands::List).unwrap_err();
        assert!(err.to_string().contains("Failed to open database"));
    }
}
