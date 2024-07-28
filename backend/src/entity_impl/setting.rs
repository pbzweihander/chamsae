use activitypub_federation::http_signatures::generate_actor_keypair;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ConnectionTrait, EntityTrait, PaginatorTrait, TransactionTrait,
};
use ulid::Ulid;

use crate::{
    entity::{sea_orm_active_enums::ObjectStoreType, setting},
    error::{Context, Error},
    format_err,
};

impl setting::Model {
    pub async fn init(
        instance_name: String,
        user_handle: String,
        user_password: String,
        object_store_local_file_system_base_path: String,
        db: &impl TransactionTrait,
    ) -> Result<Self, Error> {
        let tx = db
            .begin()
            .await
            .context_internal_server_error("failed to begin database transaction")?;

        let existing_count = setting::Entity::find_by_id(Ulid::nil())
            .count(&tx)
            .await
            .context_internal_server_error("failed to query database")?;
        if existing_count > 0 {
            return Err(format_err!(CONFLICT, "already initialized"));
        }

        let user_password_hash = bcrypt::hash(&user_password, 10)
            .context_internal_server_error("failed to hash user password")?;

        let keypair = generate_actor_keypair()
            .context_internal_server_error("failed to generate actor keypair")?;

        let setting_activemodel = setting::ActiveModel {
            id: ActiveValue::Set(Ulid::nil().into()),
            instance_name: ActiveValue::Set(instance_name),
            user_handle: ActiveValue::Set(user_handle),
            user_password_hash: ActiveValue::Set(user_password_hash),
            user_public_key: ActiveValue::Set(keypair.public_key),
            user_private_key: ActiveValue::Set(keypair.private_key),
            object_store_type: ActiveValue::Set(Some(ObjectStoreType::LocalFileSystem)),
            object_store_local_file_system_base_path: ActiveValue::Set(Some(
                object_store_local_file_system_base_path,
            )),
            ..Default::default()
        };
        let setting = setting_activemodel
            .insert(&tx)
            .await
            .context_internal_server_error("failed to insert to database")?;

        tx.commit()
            .await
            .context_internal_server_error("failed to commit database transaction")?;

        Ok(setting)
    }

    #[tracing::instrument(skip(db))]
    pub async fn get(db: &impl ConnectionTrait) -> Result<Self, Error> {
        let setting = setting::Entity::find_by_id(Ulid::nil())
            .one(db)
            .await
            .context_internal_server_error("failed to query database")?
            .context_not_found("not initialized")?;
        Ok(setting)
    }
}
