use activitypub_federation::{
    axum::json::FederationJson, config::Data, protocol::context::WithContext, traits::Object,
};
use axum::{
    extract,
    http::{header, HeaderMap},
    routing, Router,
};
use sea_orm::EntityTrait;
use ulid::Ulid;

use crate::{
    ap::NoteOrAnnounce,
    entity::{post, sea_orm_active_enums},
    error::{Context, Result},
    format_err,
    handler::frontend::RespOrFrontend,
    state::State,
};

pub fn create_router() -> Router {
    Router::new().route("/:id", routing::get(get_note))
}

#[tracing::instrument(skip(data))]
async fn get_note(
    data: Data<State>,
    extract::Path(id): extract::Path<Ulid>,
    headers: HeaderMap,
) -> Result<RespOrFrontend<FederationJson<WithContext<NoteOrAnnounce>>>> {
    if headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.starts_with("application/activity+json"))
        .unwrap_or_default()
    {
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
            Ok(RespOrFrontend::Resp(FederationJson(
                WithContext::new_default(this),
            )))
        }
    } else {
        Ok(RespOrFrontend::Frontend)
    }
}
