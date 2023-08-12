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

#[derive(Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_debug")]
    pub debug: bool,

    /// Domain of the instance. DO NOT CHANGE! e.g. `example.com`
    pub domain: String,

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

    /// Handle of the owner of this instance. e.g. `admin`
    pub user_handle: String,
    /// Password bcrypt hash of the owner user of this instance
    pub user_password_bcrypt: String,

    #[serde(skip)]
    pub user_id: Option<Url>,
    #[serde(skip)]
    pub inbox_url: Option<Url>,

    /// Public key PEM file path for the owner user of this instance
    pub user_public_key_path: PathBuf,
    /// Private key PEM file path for the owner user of this instance
    pub user_private_key_path: PathBuf,

    #[serde(skip)]
    pub user_public_key: String,
    #[serde(skip)]
    pub user_private_key: String,

    /// Region of the S3 compatible object storage.
    /// e.g. `ap-northeast-1` for AWS, `nyc3` for DigitalOcean, or `auto`
    pub object_storage_region: String,
    /// API endpoint of the S3 compatible object storage.
    /// e.g. `s3-ap-northeast-1.amazonaws.com`, or `{account_id}.r2.cloudflarestorage.com`
    pub object_storage_endpoint: String,
    /// Bucket name of the S3 compatible object storage. e.g. `my-bucket`
    pub object_storage_bucket: String,
    /// Public endpoint Base URL of the S3 compatible object storage.
    /// If the bucket is connected to a domain, this value should be that domain.
    /// If the bucket is connected to a CDN, this value should be the CDN domain.
    /// Note: trailing slash is mandatory.
    /// e.g. `https://example.com`
    pub object_storage_public_url_base: Url,
    /// Whether to enable path style for the object storage
    #[serde(default)]
    pub object_storage_path_style: bool,
    #[serde(default)]
    pub object_storage_access_key: Option<String>,
    #[serde(default)]
    pub object_storage_secret_key: Option<String>,

    #[serde(skip)]
    pub object_storage_creds: Option<s3::creds::Credentials>,
}

impl Config {
    pub fn object_storage_bucket(&self) -> Result<s3::Bucket> {
        let object_storage_bucket = s3::Bucket::new(
            &self.object_storage_bucket,
            s3::Region::Custom {
                region: self.object_storage_region.clone(),
                endpoint: self.object_storage_endpoint.clone(),
            },
            self.object_storage_creds.clone().unwrap(),
        )
        .context("failed to initialize object storage bucket")?;
        Ok(if self.object_storage_path_style {
            object_storage_bucket.with_path_style()
        } else {
            object_storage_bucket
        })
    }

    pub fn try_from_env() -> Result<Self> {
        let mut config: Config =
            envy::from_env().context("failed to parse config fro environment variables")?;

        let user_id = Url::parse(&format!("https://{}/ap/person", config.domain))
            .context("failed to construct ID URL")?;
        let inbox_url = Url::parse(&format!("https://{}/ap/inbox", config.domain))
            .context("failed to construct inbox URL")?;

        let user_public_key = std::fs::read_to_string(&config.user_public_key_path)
            .context("failed to read public key file")?;
        let user_private_key = std::fs::read_to_string(&config.user_private_key_path)
            .context("failed to read private key file")?;

        let object_storage_creds = s3::creds::Credentials::new(
            config.object_storage_access_key.as_deref(),
            config.object_storage_secret_key.as_deref(),
            None,
            None,
            None,
        )
        .context("failed to initialize object storage credential")?;
        config.user_id = Some(user_id);
        config.inbox_url = Some(inbox_url);
        config.user_public_key = user_public_key;
        config.user_private_key = user_private_key;
        config.object_storage_creds = Some(object_storage_creds);

        Ok(config)
    }
}
