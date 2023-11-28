use activitypub_federation::{
    axum::json::FederationJson, config::Data, protocol::context::WithContext, traits::Object,
};
use axum::{
    extract,
    http::{header, HeaderMap},
    routing, Router,
};
use reqwest::StatusCode;
use sea_orm::EntityTrait;
use ulid::Ulid;

use crate::{
    ap::{person::LocalPerson, NoteOrAnnounce},
    entity::{post, user},
    error::{Context, Result},
    format_err,
    handler::frontend::{FrontendContext, RespOrFrontend},
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
    let this = post::Entity::find_by_id(id)
        .one(&*data.db)
        .await
        .context_internal_server_error("failed to query database")?;
    if let Some(this) = this {
        if this.visibility.is_visible() {
            if headers
                .get(header::ACCEPT)
                .and_then(|v| v.to_str().ok())
                .map(|v| v.starts_with("application/activity+json"))
                .unwrap_or_default()
            {
                let this = this.into_json(&data).await?;
                return Ok(RespOrFrontend::resp(FederationJson(
                    WithContext::new_default(this),
                )));
            } else {
                let (name, avatar_url) = if let Some(user_id) = this.user_id {
                    let user = user::Entity::find_by_id(user_id)
                        .one(&*data.db)
                        .await
                        .context_internal_server_error("failed to query database")?
                        .context_internal_server_error("user not found")?;
                    (user.display_name().to_string(), user.avatar_url)
                } else {
                    let local_user = LocalPerson::get(&*data.db).await?;
                    (
                        local_user.display_name().to_string(),
                        local_user
                            .get_avatar_url(&*data.db)
                            .await?
                            .map(|url| url.to_string()),
                    )
                };

                let ctx = FrontendContext {
                    title: Some(name.clone()),
                    description: Some(this.text.clone()),
                    og_type: Some("article".to_string()),
                    og_title: Some(name),
                    og_description: Some(this.text),
                    og_image: avatar_url,
                };

                return RespOrFrontend::frontend(StatusCode::OK, &*data.db, ctx).await;
            }
        }
    }
    if headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.starts_with("application/activity+json"))
        .unwrap_or_default()
    {
        Err(format_err!(NOT_FOUND, "post not found"))
    } else {
        let ctx = FrontendContext::site_default(&*data.db).await?;
        RespOrFrontend::frontend(StatusCode::NOT_FOUND, &*data.db, ctx).await
    }
}
