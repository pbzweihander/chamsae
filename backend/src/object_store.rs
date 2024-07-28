use std::path::PathBuf;

use axum::body::Bytes;
use object_store::{
    aws::{AmazonS3, AmazonS3Builder},
    local::LocalFileSystem,
    path::Path,
    ObjectStore as _,
};
use url::Url;

use crate::{
    config::CONFIG,
    entity::{sea_orm_active_enums::ObjectStoreType, setting},
    error::{Context, Result},
    format_err,
};

pub struct ObjectStore {
    ty: ObjectStoreType,
    s3_store: Option<(AmazonS3, Url)>,
    local_file_system_store: Option<(LocalFileSystem, PathBuf)>,
}

impl ObjectStore {
    pub fn from_setting(setting: &setting::Model) -> Result<Self> {
        let ty = setting
            .object_store_type
            .clone()
            .context_internal_server_error("not initialized")?;

        let s3_store = if let (Some(bucket), Some(public_url_base)) = (
            &setting.object_store_s3_bucket,
            &setting.object_store_s3_public_url_base,
        ) {
            let public_url_base = Url::parse(public_url_base)
                .context_internal_server_error("malformed public URL base")?;
            Some((
                AmazonS3Builder::from_env()
                    .with_bucket_name(bucket)
                    .build()
                    .context_internal_server_error("failed to build S3 object store")?,
                public_url_base,
            ))
        } else {
            None
        };

        let local_file_system_store =
            if let Some(base_path) = &setting.object_store_local_file_system_base_path {
                let base_path = PathBuf::from(base_path.clone());
                Some((
                    LocalFileSystem::new_with_prefix(&base_path).context_internal_server_error(
                        "failed to build local filesystem object store",
                    )?,
                    base_path,
                ))
            } else {
                None
            };

        if !((ty == ObjectStoreType::S3 && s3_store.is_some())
            || (ty == ObjectStoreType::LocalFileSystem && local_file_system_store.is_some()))
        {
            return Err(format_err!(
                INTERNAL_SERVER_ERROR,
                "invalid object store setting"
            ));
        }

        Ok(Self {
            ty,
            s3_store,
            local_file_system_store,
        })
    }

    /// Returns saved key, type, and public URL
    pub async fn put(&self, key: &str, body: Bytes) -> Result<(String, ObjectStoreType, Url)> {
        let path = Path::parse(key)
            .context_internal_server_error("failed to construct object store key")?;
        let res = match &self.ty {
            ObjectStoreType::S3 => {
                let (store, public_url_base) = self.s3_store.as_ref().unwrap();
                store
                    .put(&path, body.into())
                    .await
                    .context_internal_server_error("failed to put object to object store")?;
                let url = public_url_base
                    .join(key)
                    .context_internal_server_error("failed to construct object public URL")?;
                (key.to_string(), ObjectStoreType::S3, url)
            }
            ObjectStoreType::LocalFileSystem => {
                let (store, base_path) = self.local_file_system_store.as_ref().unwrap();
                store
                    .put(&path, body.into())
                    .await
                    .context_internal_server_error("failed to put object to object store")?;
                (
                    base_path.join(key).to_string_lossy().to_string(),
                    ObjectStoreType::LocalFileSystem,
                    Url::parse(&format!("https://{}/file/{}", CONFIG.public_domain, key))
                        .context_internal_server_error("failed to construct public URL")?,
                )
            }
        };
        Ok(res)
    }

    pub async fn delete(&self, key: &str, ty: &ObjectStoreType) -> Result<()> {
        match ty {
            ObjectStoreType::S3 => {
                let (s3_store, _) = self
                    .s3_store
                    .as_ref()
                    .context_internal_server_error("S3 object store setting not found")?;
                let path =
                    Path::parse(key).context_internal_server_error("malfored object store key")?;
                s3_store
                    .delete(&path)
                    .await
                    .context_internal_server_error("failed to delete object from object store")?;
            }
            ObjectStoreType::LocalFileSystem => {
                tokio::fs::remove_file(key)
                    .await
                    .context_internal_server_error(
                        "failed to delete object from local filesystem",
                    )?;
            }
        }
        Ok(())
    }
}
