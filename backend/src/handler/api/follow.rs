use activitypub_federation::{config::Data, fetch::object_id::ObjectId, traits::Object};
use axum::{extract, routing, Json, Router};
use sea_orm::{
    ActiveModelTrait, ActiveValue, EntityTrait, ModelTrait, PaginatorTrait, TransactionTrait,
};
use ulid::Ulid;
use url::Url;

use crate::{
    ap::{follow::Follow, undo::Undo},
    dto::CreateFollow,
    entity::{follow, user},
    error::{Context, Result},
    format_err,
    state::State,
};

use super::auth::Access;

pub(super) fn create_router() -> Router {
    Router::new()
        .route("/", routing::post(post_follow))
        .route("/:id", routing::delete(delete_follow))
}

#[tracing::instrument(skip(data, _access))]
async fn post_follow(
    data: Data<State>,
    _access: Access,
    Json(req): Json<CreateFollow>,
) -> Result<()> {
    let tx = data
        .db
        .begin()
        .await
        .context_internal_server_error("failed to begin database transaction")?;

    let user_existing_count = user::Entity::find_by_id(req.to_id)
        .count(&tx)
        .await
        .context_internal_server_error("failed to query database")?;

    if user_existing_count == 0 {
        return Err(format_err!(NOT_FOUND, "user not found"));
    }

    let existing_count = follow::Entity::find_by_id(req.to_id)
        .count(&tx)
        .await
        .context_internal_server_error("failed to query database")?;

    if existing_count != 0 {
        return Ok(());
    }

    let follow_activemodel = follow::ActiveModel {
        to_id: ActiveValue::Set(req.to_id.into()),
        accepted: ActiveValue::Set(false),
    };
    let follow = follow_activemodel
        .insert(&tx)
        .await
        .context_internal_server_error("failed to insert to database")?;

    tx.commit()
        .await
        .context_internal_server_error("failed to commit database transaction")?;

    let follow = follow.into_json(&data).await?;
    follow.send(&data).await?;

    Ok(())
}

#[tracing::instrument(skip(data, _access))]
async fn delete_follow(
    data: Data<State>,
    extract::Path(id): extract::Path<Ulid>,
    _access: Access,
) -> Result<()> {
    let tx = data
        .db
        .begin()
        .await
        .context_internal_server_error("failed to begin database transaction")?;

    let existing = follow::Entity::find_by_id(id)
        .one(&tx)
        .await
        .context_internal_server_error("failed to query database")?;

    if let Some(existing) = existing {
        let ap = existing.clone().into_json(&data).await?;

        ModelTrait::delete(existing, &tx)
            .await
            .context_internal_server_error("failed to delete from database")?;

        tx.commit()
            .await
            .context_internal_server_error("failed to commit database transaction")?;

        let object_id: ObjectId<user::Model> = ap.object.clone().into();
        let object = object_id.dereference(&data).await?;
        let inbox =
            Url::parse(&object.inbox).context_internal_server_error("malformed user inbox URL")?;
        let undo = Undo::<Follow, follow::Model>::new(ap)?;
        undo.send(&data, vec![inbox]).await?;

        Ok(())
    } else {
        Ok(())
    }
}
