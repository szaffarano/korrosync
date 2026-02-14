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
//!
//! ### TLS Configuration (when `tls` feature is enabled)
//! - `KORROSYNC_USE_TLS` - Enable TLS/HTTPS support (default: `false`)
//!   - Accepts: `true`, `1`, `yes`, `on` (case-insensitive)
//!   - Rejects: `false`, `0`, `no`, `off` (case-insensitive)
//! - `KORROSYNC_CERT_PATH` - Path to TLS certificate file in PEM format (default: `tls/cert.pem`)
//! - `KORROSYNC_KEY_PATH` - Path to TLS private key file in PEM format (default: `tls/key.pem`)
//!
//! ## Rate Limiting
//! - `KORROSYNC_RATE_LIMIT_PER_SECOND` - Rate limit replenishment rate per second (default: `2`)
//! - `KORROSYNC_RATE_LIMIT_BURST_SIZE` - Maximum burst size before rate limiting (default: `5`)

use std::env;

use serde::{Deserialize, Serialize};

const DEFAULT_DB_PATH: &str = "data/db.redb";
const DEFAULT_SERVER_ADDRESS: &str = "0.0.0.0:3000";
#[cfg(feature = "tls")]
const DEFAULT_TLS_CERT: &str = "tls/cert.pem";
#[cfg(feature = "tls")]
const DEFAULT_TLS_PRIVKEY: &str = "tls/key.pem";
const DEFAULT_RATE_LIMIT_PER_SECOND: u64 = 2;
const DEFAULT_RATE_LIMIT_BURST_SIZE: u32 = 5;

/// Main configuration structure for Korrosync
///
/// Contains all configuration settings loaded from environment variables.
#[derive(Serialize, Deserialize)]
pub struct Config {
    /// Database configuration
    pub db: Db,
    /// Server configuration including TLS settings
    pub server: Server,
    /// Rate limiting configuration
    pub rate_limit: RateLimit,
}

/// Database configuration
#[derive(Serialize, Deserialize)]
pub struct Db {
    /// Path to the redb database file
    pub path: String,
}

/// Server configuration
#[cfg_attr(feature = "tls", doc = "including TLS settings")]
#[derive(Serialize, Deserialize)]
pub struct Server {
    /// Server bind address (e.g., "0.0.0.0:3000")
    pub address: String,
    /// Path to TLS certificate file in PEM format
    #[cfg(feature = "tls")]
    pub cert_path: String,
    /// Path to TLS private key file in PEM format
    #[cfg(feature = "tls")]
    pub key_path: String,
    /// Whether to enable TLS/HTTPS
    ///
    /// When `true`, the server will use TLS with the configured certificate and key.
    /// Supports multiple boolean representations: true/1/yes/on or false/0/no/off (case-insensitive)
    #[cfg(feature = "tls")]
    pub use_tls: bool,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            db: Db::from_env(),
            server: Server::from_env(),
            rate_limit: RateLimit::from_env(),
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

        #[cfg(feature = "tls")]
        let cert_path = env::var("KORROSYNC_CERT_PATH").unwrap_or(DEFAULT_TLS_CERT.to_string());
        #[cfg(feature = "tls")]
        let key_path = env::var("KORROSYNC_KEY_PATH").unwrap_or(DEFAULT_TLS_PRIVKEY.to_string());
        #[cfg(feature = "tls")]
        let use_tls_str = env::var("KORROSYNC_USE_TLS").unwrap_or("false".to_string());
        #[cfg(feature = "tls")]
        let use_tls = match use_tls_str.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => true,
            "false" | "0" | "no" | "off" => false,
            _ => panic!(
                "Invalid boolean value for KORROSYNC_USE_TLS: '{}'. Expected: true/1/yes/on or false/0/no/off",
                use_tls_str
            ),
        };

        Self {
            address,
            #[cfg(feature = "tls")]
            cert_path,
            #[cfg(feature = "tls")]
            key_path,
            #[cfg(feature = "tls")]
            use_tls,
        }
    }
}

/// Rate limiting configuration
#[derive(Serialize, Deserialize)]
pub struct RateLimit {
    /// Replenishment rate in requests per second
    pub per_second: u64,
    /// Maximum burst size before rate limiting kicks in
    pub burst_size: u32,
}

impl RateLimit {
    pub fn from_env() -> Self {
        let per_second = env::var("KORROSYNC_RATE_LIMIT_PER_SECOND")
            .map(|v| {
                v.parse::<u64>().unwrap_or_else(|_| {
                    panic!(
                        "Invalid value for KORROSYNC_RATE_LIMIT_PER_SECOND: '{}'. Expected a positive integer",
                        v
                    )
                })
            })
            .unwrap_or(DEFAULT_RATE_LIMIT_PER_SECOND);

        assert!(
            per_second > 0,
            "KORROSYNC_RATE_LIMIT_PER_SECOND must be greater than 0"
        );

        let burst_size = env::var("KORROSYNC_RATE_LIMIT_BURST_SIZE")
            .map(|v| {
                v.parse::<u32>().unwrap_or_else(|_| {
                    panic!(
                        "Invalid value for KORROSYNC_RATE_LIMIT_BURST_SIZE: '{}'. Expected a positive integer",
                        v
                    )
                })
            })
            .unwrap_or(DEFAULT_RATE_LIMIT_BURST_SIZE);

        assert!(
            burst_size > 0,
            "KORROSYNC_RATE_LIMIT_BURST_SIZE must be greater than 0"
        );

        Self {
            per_second,
            burst_size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limit_defaults() {
        temp_env::with_vars_unset(
            vec![
                "KORROSYNC_RATE_LIMIT_PER_SECOND",
                "KORROSYNC_RATE_LIMIT_BURST_SIZE",
            ],
            || {
                let rate_limit = RateLimit::from_env();
                assert_eq!(rate_limit.per_second, 2);
                assert_eq!(rate_limit.burst_size, 5);
            },
        );
    }

    #[test]
    fn rate_limit_custom_values() {
        temp_env::with_vars(
            vec![
                ("KORROSYNC_RATE_LIMIT_PER_SECOND", Some("10")),
                ("KORROSYNC_RATE_LIMIT_BURST_SIZE", Some("20")),
            ],
            || {
                let rate_limit = RateLimit::from_env();
                assert_eq!(rate_limit.per_second, 10);
                assert_eq!(rate_limit.burst_size, 20);
            },
        );
    }

    #[test]
    #[should_panic(expected = "Invalid value for KORROSYNC_RATE_LIMIT_PER_SECOND")]
    fn rate_limit_invalid_per_second() {
        temp_env::with_vars(
            vec![("KORROSYNC_RATE_LIMIT_PER_SECOND", Some("abc"))],
            || {
                RateLimit::from_env();
            },
        );
    }

    #[test]
    #[should_panic(expected = "Invalid value for KORROSYNC_RATE_LIMIT_BURST_SIZE")]
    fn rate_limit_invalid_burst_size() {
        temp_env::with_vars(
            vec![("KORROSYNC_RATE_LIMIT_BURST_SIZE", Some("xyz"))],
            || {
                RateLimit::from_env();
            },
        );
    }

    #[test]
    #[should_panic(expected = "KORROSYNC_RATE_LIMIT_PER_SECOND must be greater than 0")]
    fn rate_limit_zero_per_second() {
        temp_env::with_vars(vec![("KORROSYNC_RATE_LIMIT_PER_SECOND", Some("0"))], || {
            RateLimit::from_env();
        });
    }

    #[test]
    #[should_panic(expected = "KORROSYNC_RATE_LIMIT_BURST_SIZE must be greater than 0")]
    fn rate_limit_zero_burst_size() {
        temp_env::with_vars(vec![("KORROSYNC_RATE_LIMIT_BURST_SIZE", Some("0"))], || {
            RateLimit::from_env();
        });
    }
}
