use std::path::PathBuf;

use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use serde::Deserialize;
use url::Url;

pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    Config::try_from_env().expect("failed to parse config from environment variables")
});

fn default_listen_addr() -> String {
    "0.0.0.0:3000".to_string()
}

fn default_database_host() -> String {
    "localhost".to_string()
}

fn default_database_port() -> u16 {
    5432
}

fn default_database_user() -> String {
    "postgres".to_string()
}

fn default_database_password() -> String {
    "chamsae".to_string()
}

fn default_database_database() -> String {
    "postgres".to_string()
}

fn default_static_files_directory_path() -> Option<PathBuf> {
    Some(PathBuf::from("../frontend/dist"))
}

#[derive(Clone, Deserialize)]
pub struct Config {
    pub domain: Url,

    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,

    #[serde(default = "default_database_host")]
    pub database_host: String,
    #[serde(default = "default_database_port")]
    pub database_port: u16,
    #[serde(default = "default_database_user")]
    pub database_user: String,
    #[serde(default = "default_database_password")]
    pub database_password: String,
    #[serde(default = "default_database_database")]
    pub database_database: String,

    /// Path to the static frontend files directory.
    /// Set to None if you want to serve frontend in another method (e.g. CDN)
    #[serde(default = "default_static_files_directory_path")]
    pub static_files_directory_path: Option<PathBuf>,

    /// Handle of the owner of this instance
    pub user_handle: String,
    /// Password bcrypt hash of the owner user of this instance
    pub user_password_bcrypt: String,
}

impl Config {
    pub fn try_from_env() -> Result<Self> {
        let config: Config =
            envy::from_env().context("failed to parse config fro environment variables")?;
        Ok(config)
    }
}
