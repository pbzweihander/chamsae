use activitypub_federation::{
    axum::json::FederationJson, config::Data, protocol::context::WithContext, traits::Object,
};
use axum::{extract, routing, Router};
use sea_orm::EntityTrait;
use ulid::Ulid;

use crate::{
    ap::NoteOrAnnounce,
    entity::{post, sea_orm_active_enums},
    error::{Context, Result},
    format_err,
    state::State,
};

pub(super) fn create_router() -> Router {
    Router::new().route("/:id", routing::get(get_note))
}

#[tracing::instrument(skip(data))]
async fn get_note(
    data: Data<State>,
    extract::Path(id): extract::Path<Ulid>,
) -> Result<FederationJson<WithContext<NoteOrAnnounce>>> {
    let this = post::Entity::find_by_id(id)
        .one(&*data.db)
        .await
        .context_internal_server_error("failed to query database")?
        .context_not_found("post not found")?;
    if this.visibility == sea_orm_active_enums::Visibility::Followers
        || this.visibility == sea_orm_active_enums::Visibility::DirectMessage
    {
        Err(format_err!(NOT_FOUND, "post not found"))
    } else {
        let this = this.into_json(&data).await?;
        Ok(FederationJson(WithContext::new_default(this)))
    }
}
