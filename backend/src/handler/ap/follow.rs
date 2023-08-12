use activitypub_federation::{
    axum::json::FederationJson, config::Data, protocol::context::WithContext, traits::Object,
};
use axum::{extract, routing, Router};
use sea_orm::EntityTrait;
use ulid::Ulid;

use crate::{
    ap::follow::Follow,
    entity::follow,
    error::{Context, Result},
    state::State,
};

pub(super) fn create_router() -> Router {
    Router::new().route("/:id", routing::get(get_follow))
}

#[tracing::instrument(skip(data))]
async fn get_follow(
    data: Data<State>,
    extract::Path(id): extract::Path<Ulid>,
) -> Result<FederationJson<WithContext<Follow>>> {
    let this = follow::Entity::find_by_id(id)
        .one(&*data.db)
        .await
        .context_internal_server_error("failed to query database")?
        .context_not_found("post not found")?;
    let this = this.into_json(&data).await?;
    Ok(FederationJson(WithContext::new_default(this)))
}
