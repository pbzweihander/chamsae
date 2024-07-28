use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use serde::Deserialize;
use url::Url;

pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    Config::try_from_env().expect("failed to parse config from environment variables")
});

fn default_debug() -> bool {
    false
}

fn default_public_domain() -> String {
    "localhost".to_string()
}

fn default_listen_addr() -> String {
    "0.0.0.0:3000".to_string()
}

fn default_database_url() -> Url {
    Url::parse("postgresql://postgres:chamsae@localhost:5432").unwrap()
}

#[derive(Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_debug")]
    pub debug: bool,

    /// Public domain of the instance.
    /// DO NOT CHANGE!
    /// e.g. `example.com`
    #[serde(default = "default_public_domain")]
    pub public_domain: String,

    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,

    #[serde(default = "default_database_url")]
    pub database_url: Url,
}

impl Config {
    pub fn try_from_env() -> Result<Self> {
        let config: Config =
            envy::from_env().context("failed to parse config from environment variables")?;
        Ok(config)
    }
}
