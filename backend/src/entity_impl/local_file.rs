use axum::body::Bytes;
use migration::ConnectionTrait;
use mime::Mime;
use sea_orm::{ActiveModelTrait, ActiveValue, ModelTrait};
use ulid::Ulid;

use crate::{
    config::CONFIG,
    entity::local_file,
    error::{Context, Result},
    format_err,
};

impl local_file::Model {
    #[tracing::instrument(skip(data, db))]
    pub async fn new(
        data: Bytes,
        media_type: Mime,
        alt: Option<String>,
        db: &impl ConnectionTrait,
    ) -> Result<Self> {
        let id = Ulid::new();

        let bucket = CONFIG.object_storage_bucket()?;
        let object_storage_key = id.to_string();
        let res = bucket
            .put_object_with_content_type(&object_storage_key, &data, media_type.as_ref())
            .await
            .context_internal_server_error("failed to put object to object storage")?;
        if res.status_code() >= 400 {
            if let Ok(res_str) = res.to_string() {
                return Err(format_err!(
                    INTERNAL_SERVER_ERROR,
                    "failed to put object to object storage, status code: {}, message: {}",
                    res.status_code(),
                    res_str
                ));
            } else {
                return Err(format_err!(
                    INTERNAL_SERVER_ERROR,
                    "failed to put object to object storage, status code: {}",
                    res.status_code()
                ));
            }
        }
        let url = CONFIG
            .object_storage_public_url_base
            .join(&object_storage_key)
            .context_internal_server_error("failed to construct object public URL")?;

        let this = Self {
            id: id.into(),
            post_id: None,
            emoji_name: None,
            order: None,
            object_storage_key,
            media_type: media_type.to_string(),
            url: url.to_string(),
            alt,
        };
        let this_activemodel: local_file::ActiveModel = this.into();
        let this = this_activemodel
            .insert(db)
            .await
            .context_internal_server_error("failed to insert to database")?;

        Ok(this)
    }

    #[tracing::instrument(skip(db))]
    pub async fn attach_to_post(
        &self,
        post_id: Ulid,
        order: u8,
        db: &impl ConnectionTrait,
    ) -> Result<()> {
        let this_activemodel = local_file::ActiveModel {
            id: ActiveValue::Unchanged(self.id),
            post_id: ActiveValue::Set(Some(post_id.into())),
            order: ActiveValue::Set(Some(order as i16)),
            ..Default::default()
        };
        this_activemodel
            .update(db)
            .await
            .context_internal_server_error("failed to update database")?;
        Ok(())
    }

    #[tracing::instrument(skip(db))]
    pub async fn attach_to_emoji(
        &self,
        emoji_name: String,
        db: &impl ConnectionTrait,
    ) -> Result<()> {
        let this_activemodel = local_file::ActiveModel {
            id: ActiveValue::Unchanged(self.id),
            emoji_name: ActiveValue::Set(Some(emoji_name)),
            ..Default::default()
        };
        this_activemodel
            .update(db)
            .await
            .context_internal_server_error("failed to update database")?;
        Ok(())
    }

    #[tracing::instrument(skip(db))]
    pub async fn delete(self, db: &impl ConnectionTrait) -> Result<()> {
        let bucket = CONFIG.object_storage_bucket()?;
        let res = bucket
            .delete_object(&self.object_storage_key)
            .await
            .context_internal_server_error("failed to delete object from object storage")?;
        if res.status_code() >= 500 {
            if let Ok(res_str) = res.to_string() {
                return Err(format_err!(
                    INTERNAL_SERVER_ERROR,
                    "failed to delete object from object storage, status code: {}, message: {}",
                    res.status_code(),
                    res_str
                ));
            } else {
                return Err(format_err!(
                    INTERNAL_SERVER_ERROR,
                    "failed to delete object from object storage, status code: {}",
                    res.status_code()
                ));
            }
        }

        ModelTrait::delete(self, db)
            .await
            .context_internal_server_error("failed to delete from database")?;

        Ok(())
    }
}
