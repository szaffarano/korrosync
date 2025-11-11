use std::env;

use serde::{Deserialize, Serialize};

const DEFAULT_DB_PATH: &str = "data/db.redb";
const DEFAULT_SERVER_ADDRESS: &str = "0.0.0.0:3000";

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
        Self { address }
    }
}
