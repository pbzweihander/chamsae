use std::path::PathBuf;

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

fn default_listen_addr() -> String {
    "0.0.0.0:3000".to_string()
}

fn default_database_url() -> Url {
    Url::parse("postgresql://postgres:chamsae@localhost:5432").unwrap()
}

fn default_object_store_local_file_base_path() -> PathBuf {
    PathBuf::from("./files/")
}

#[derive(Clone, Deserialize)]
pub struct ObjectStorageS3Config {
    /// Bucket name of the S3 compatible object storage. e.g. `my-bucket`
    pub object_store_bucket: String,
    /// Public endpoint Base URL of the S3 compatible object storage.
    /// If the bucket is connected to a domain, this value should be that domain.
    /// If the bucket is connected to a CDN, this value should be the CDN domain.
    /// Note: trailing slash is mandatory.
    /// e.g. `https://example.com`
    pub object_store_public_url_base: Url,
}

#[derive(Clone, Deserialize)]
pub struct ObjectStorageLocalFilesystemConfig {
    /// Directory path for the local files to be stored. e.g. `./files/`
    #[serde(default = "default_object_store_local_file_base_path")]
    pub object_store_local_file_base_path: PathBuf,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "snake_case", tag = "object_store_type")]
pub enum ObjectStoreConfig {
    /// With S3 option, you can provide following environment variables to config:
    /// - `AWS_ACCESS_KEY_ID`,
    /// - `AWS_SECRET_ACCESS_KEY`
    /// - `AWS_DEFAULT_REGION`
    /// - `AWS_ENDPOINT`
    /// - `AWS_SESSION_TOKEN`
    /// - `AWS_CONTAINER_CREDENTIALS_RELATIVE_URI`
    /// - `AWS_ALLOW_HTTP`
    /// Reference: https://docs.rs/object_store/latest/object_store/aws/struct.AmazonS3Builder.html#method.from_env
    S3(ObjectStorageS3Config),
    LocalFilesystem(ObjectStorageLocalFilesystemConfig),
}

#[derive(Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_debug")]
    pub debug: bool,

    /// Domain of the instance.
    /// DO NOT CHANGE!
    /// e.g. `example.com`
    pub domain: String,

    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,

    #[serde(default = "default_database_url")]
    pub database_url: Url,

    #[serde(flatten)]
    pub object_store_config: ObjectStoreConfig,
}

impl Config {
    pub fn try_from_env() -> Result<Self> {
        let config: Config =
            envy::from_env().context("failed to parse config from environment variables")?;
        Ok(config)
    }
}
