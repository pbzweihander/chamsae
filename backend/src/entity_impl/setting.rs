use activitypub_federation::http_signatures::generate_actor_keypair;
use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait, TransactionTrait};
use ulid::Ulid;

use crate::{
    entity::setting,
    error::{Context, Error},
};

impl setting::Model {
    #[tracing::instrument(skip(db))]
    pub async fn get(db: &impl TransactionTrait) -> Result<Self, Error> {
        let tx = db
            .begin()
            .await
            .context_internal_server_error("failed to begin database transaction")?;

        let setting = setting::Entity::find_by_id(Ulid::nil())
            .one(&tx)
            .await
            .context_internal_server_error("failed to query database")?;

        let setting = if let Some(setting) = setting {
            setting
        } else {
            let keypair = generate_actor_keypair()
                .context_internal_server_error("failed to generate actor keypair")?;

            let setting_activemodel = setting::ActiveModel {
                id: ActiveValue::Set(Ulid::nil().into()),
                user_public_key: ActiveValue::Set(keypair.public_key),
                user_private_key: ActiveValue::Set(keypair.private_key),
                ..Default::default()
            };
            let setting = setting_activemodel
                .insert(&tx)
                .await
                .context_internal_server_error("failed to insert to database")?;

            tx.commit()
                .await
                .context_internal_server_error("failed to commit database transaction")?;

            setting
        };

        Ok(setting)
    }
}
