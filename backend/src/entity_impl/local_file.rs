use axum::body::Bytes;
use migration::ConnectionTrait;
use mime::Mime;
use sea_orm::{ActiveModelTrait, ActiveValue, ModelTrait};
use uuid::Uuid;

use crate::{
    entity::local_file,
    error::{Context, Result},
};

impl local_file::Model {
    #[tracing::instrument(skip(data, db))]
    pub async fn new(
        data: Bytes,
        media_type: Mime,
        alt: Option<String>,
        db: &impl ConnectionTrait,
    ) -> Result<Self> {
        let id = Uuid::new_v4();

        // TODO: upload to object storage
        let _ = data;
        let object_storage_key = "example/example.png".to_string();
        let url = "https://fastly.picsum.photos/id/472/200/200.jpg?hmac=PScxKeNxgxcauarhbWIWesyo4VsouCtfdX8fNTy9HRI".to_string();

        let this = Self {
            id,
            post_id: None,
            order: None,
            object_storage_key,
            media_type: media_type.to_string(),
            url,
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
        post_id: Uuid,
        order: u8,
        db: &impl ConnectionTrait,
    ) -> Result<()> {
        let this_activemodel = local_file::ActiveModel {
            id: ActiveValue::Unchanged(self.id),
            post_id: ActiveValue::Set(Some(post_id)),
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
    pub async fn delete(self, db: &impl ConnectionTrait) -> Result<()> {
        // TODO: delete from object storage

        ModelTrait::delete(self, db)
            .await
            .context_internal_server_error("failed to delete from database")?;

        Ok(())
    }
}
