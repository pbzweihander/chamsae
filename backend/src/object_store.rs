use axum::body::Bytes;
use object_store::{aws::AmazonS3Builder, local::LocalFileSystem, path::Path, DynObjectStore};
use once_cell::sync::Lazy;
use url::Url;

use crate::{
    config::{Config, ObjectStoreConfig, CONFIG},
    entity::sea_orm_active_enums,
    error::{Context, Result},
    format_err,
};

pub static OBJECT_STORE: Lazy<ObjectStore> =
    Lazy::new(|| ObjectStore::from_config(&CONFIG).expect("failed to build object store"));

pub struct ObjectStore {
    inner: Box<DynObjectStore>,
    config: ObjectStoreConfig,
}

impl ObjectStore {
    fn from_config(config: &Config) -> anyhow::Result<Self> {
        let inner: Box<DynObjectStore> = match &config.object_store_config {
            ObjectStoreConfig::S3(config) => {
                let store = anyhow::Context::context(
                    AmazonS3Builder::from_env()
                        .with_bucket_name(&config.object_store_bucket)
                        .build(),
                    "failed to build S3 object store",
                )?;
                Box::new(store)
            }
            ObjectStoreConfig::LocalFilesystem(config) => {
                let store = anyhow::Context::context(
                    LocalFileSystem::new_with_prefix(&config.object_store_local_file_base_path),
                    "directory does not exists",
                )?;
                Box::new(store)
            }
        };
        Ok(Self {
            inner,
            config: config.object_store_config.clone(),
        })
    }

    /// Returns saved key, type, and public URL
    pub async fn put(
        &self,
        key: &str,
        body: Bytes,
    ) -> Result<(String, sea_orm_active_enums::ObjectStoreType, Url)> {
        let path = Path::parse(key)
            .context_internal_server_error("failed to construct object store key")?;
        self.inner
            .put(&path, body.into())
            .await
            .context_internal_server_error("failed to put object to object store")?;
        Ok(match &self.config {
            ObjectStoreConfig::S3(config) => {
                let url = config
                    .object_store_public_url_base
                    .join(key)
                    .context_internal_server_error("failed to construct object public URL")?;
                (
                    key.to_string(),
                    sea_orm_active_enums::ObjectStoreType::S3,
                    url,
                )
            }
            ObjectStoreConfig::LocalFilesystem(config) => (
                config
                    .object_store_local_file_base_path
                    .join(key)
                    .to_string_lossy()
                    .to_string(),
                sea_orm_active_enums::ObjectStoreType::LocalFileSystem,
                Url::parse(&format!("https://{}/file/{}", CONFIG.domain, key))
                    .context_internal_server_error("failed to construct public URL")?,
            ),
        })
    }

    pub async fn delete(
        &self,
        key: &str,
        ty: &sea_orm_active_enums::ObjectStoreType,
    ) -> Result<()> {
        match ty {
            sea_orm_active_enums::ObjectStoreType::S3 => {
                if let ObjectStoreConfig::S3(_) = &self.config {
                    let path = Path::parse(key)
                        .context_internal_server_error("malfored object store key")?;
                    self.inner
                        .delete(&path)
                        .await
                        .context_internal_server_error(
                            "failed to delete object from object store",
                        )?;
                    Ok(())
                } else {
                    Err(format_err!(
                        INTERNAL_SERVER_ERROR,
                        "cannot delete S3 stored object with local filesystem config"
                    ))
                }
            }
            sea_orm_active_enums::ObjectStoreType::LocalFileSystem => {
                tokio::fs::remove_file(key)
                    .await
                    .context_internal_server_error(
                        "failed to delete object from local filesystem",
                    )?;
                Ok(())
            }
        }
    }
}
