use axum::body::Bytes;
use mime::Mime;
use sea_orm::{ActiveModelTrait, ActiveValue, ConnectionTrait, ModelTrait};
use ulid::Ulid;

use crate::{
    entity::{local_file, sea_orm_active_enums},
    error::{Context, Result},
    object_store::OBJECT_STORE,
};

impl local_file::Model {
    #[tracing::instrument(skip(data, db))]
    pub async fn put(
        data: Bytes,
        media_type: Mime,
        alt: Option<String>,
        db: &impl ConnectionTrait,
    ) -> Result<Self> {
        let id = Ulid::new();

        let (object_store_key, object_store_type, url) =
            OBJECT_STORE.put(&id.to_string(), data).await?;

        let this_activemodel = local_file::ActiveModel {
            id: ActiveValue::Set(id.into()),
            post_id: ActiveValue::Set(None),
            emoji_name: ActiveValue::Set(None),
            order: ActiveValue::Set(None),
            object_store_key: ActiveValue::Set(object_store_key),
            object_store_type: ActiveValue::Set(object_store_type),
            media_type: ActiveValue::Set(media_type.to_string()),
            url: ActiveValue::Set(url.to_string()),
            alt: ActiveValue::Set(alt),
        };
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
        OBJECT_STORE
            .delete(&self.object_store_key, &self.object_store_type)
            .await?;

        ModelTrait::delete(self, db)
            .await
            .context_internal_server_error("failed to delete from database")?;

        Ok(())
    }

    pub fn is_local(&self) -> bool {
        self.object_store_type == sea_orm_active_enums::ObjectStoreType::LocalFileSystem
    }
}
