use activitypub_federation::{
    axum::json::FederationJson, config::Data, protocol::context::WithContext, traits::Object,
};
use axum::{extract, routing, Router};
use sea_orm::EntityTrait;
use uuid::Uuid;

use crate::{
    ap::note::Note,
    entity::post,
    error::{Context, Result},
    state::State,
};

pub(super) fn create_router() -> Router {
    Router::new().route("/:id", routing::get(get_note))
}

#[tracing::instrument(skip(data))]
async fn get_note(
    data: Data<State>,
    extract::Path(id): extract::Path<Uuid>,
) -> Result<FederationJson<WithContext<Note>>> {
    let this = post::Entity::find_by_id(id)
        .one(&*data.db)
        .await
        .context_internal_server_error("failed to query database")?
        .context_not_found("post not found")?;
    let this = this.into_json(&data).await?;
    Ok(FederationJson(WithContext::new_default(this)))
}
