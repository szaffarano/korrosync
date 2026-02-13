use clap::{Parser, Subcommand};

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

#[derive(Subcommand)]
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

#[derive(Subcommand)]
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
