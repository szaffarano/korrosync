use std::env;

use serde::{Deserialize, Serialize};

const DEFAULT_DB_PATH: &str = "data/db.redb";
const DEFAULT_SERVER_ADDRESS: &str = "0.0.0.0:3000";
const DEFAULT_TLS_CERT: &str = "tls/cert.pem";
const DEFAULT_TLS_PRIVKEY: &str = "tls/key.pem";

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub db: Db,
    pub server: Server,
}

#[derive(Serialize, Deserialize)]
pub struct Db {
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct Server {
    pub address: String,
    pub cert_path: String,
    pub key_path: String,
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
