//! Configuration management for Korrosync
//!
//! This module handles loading and managing configuration from environment variables.
//!
//! # Environment Variables
//!
//! ## Database Configuration
//! - `KORROSYNC_DB_PATH` - Path to the redb database file (default: `data/db.redb`)
//!
//! ## Server Configuration
//! - `KORROSYNC_SERVER_ADDRESS` - Server bind address (default: `0.0.0.0:3000`)
//! - `KORROSYNC_USE_TLS` - Enable TLS/HTTPS support (default: `false`)
//!   - Accepts: `true`, `1`, `yes`, `on` (case-insensitive)
//!   - Rejects: `false`, `0`, `no`, `off` (case-insensitive)
//! - `KORROSYNC_CERT_PATH` - Path to TLS certificate file in PEM format (default: `tls/cert.pem`)
//! - `KORROSYNC_KEY_PATH` - Path to TLS private key file in PEM format (default: `tls/key.pem`)

use std::env;

use serde::{Deserialize, Serialize};

const DEFAULT_DB_PATH: &str = "data/db.redb";
const DEFAULT_SERVER_ADDRESS: &str = "0.0.0.0:3000";
const DEFAULT_TLS_CERT: &str = "tls/cert.pem";
const DEFAULT_TLS_PRIVKEY: &str = "tls/key.pem";

/// Main configuration structure for Korrosync
///
/// Contains all configuration settings loaded from environment variables.
#[derive(Serialize, Deserialize)]
pub struct Config {
    /// Database configuration
    pub db: Db,
    /// Server configuration including TLS settings
    pub server: Server,
}

/// Database configuration
#[derive(Serialize, Deserialize)]
pub struct Db {
    /// Path to the redb database file
    pub path: String,
}

/// Server configuration including TLS settings
#[derive(Serialize, Deserialize)]
pub struct Server {
    /// Server bind address (e.g., "0.0.0.0:3000")
    pub address: String,
    /// Path to TLS certificate file in PEM format
    pub cert_path: String,
    /// Path to TLS private key file in PEM format
    pub key_path: String,
    /// Whether to enable TLS/HTTPS
    ///
    /// When `true`, the server will use TLS with the configured certificate and key.
    /// Supports multiple boolean representations: true/1/yes/on or false/0/no/off (case-insensitive)
    pub use_tls: bool,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            db: Db::from_env(),
            server: Server::from_env(),
        }
    }
}

impl Db {
    pub fn from_env() -> Self {
        let path = env::var("KORROSYNC_DB_PATH").unwrap_or(DEFAULT_DB_PATH.to_string());
        Self { path }
    }
}

impl Server {
    pub fn from_env() -> Self {
        let address =
            env::var("KORROSYNC_SERVER_ADDRESS").unwrap_or(DEFAULT_SERVER_ADDRESS.to_string());
        let cert_path = env::var("KORROSYNC_CERT_PATH")
            .map(|v| v.to_string())
            .unwrap_or(DEFAULT_TLS_CERT.to_string());
        let key_path = env::var("KORROSYNC_KEY_PATH")
            .map(|v| v.to_string())
            .unwrap_or(DEFAULT_TLS_PRIVKEY.to_string());
        let use_tls = env::var("KORROSYNC_USE_TLS").unwrap_or("false".to_string());
        let use_tls = match use_tls.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => true,
            "false" | "0" | "no" | "off" => false,
            _ => panic!("Invalid boolean value"),
        };
        Self {
            address,
            cert_path,
            key_path,
            use_tls,
        }
    }
}
