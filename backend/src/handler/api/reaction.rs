use activitypub_federation::config::Data;
use axum::{extract, routing, Json, Router};
use sea_orm::{EntityTrait, ModelTrait};
use ulid::Ulid;

use crate::{
    dto::Reaction,
    entity::{reaction, user},
    error::{Context, Result},
    state::State,
};

use super::auth::Access;

pub(super) fn create_router() -> Router {
    Router::new().route("/:id", routing::get(get_reaction))
}

#[utoipa::path(
    get,
    path = "/api/post/{post_id}/reaction/{reaction_id}",
    params(
        ("post_id" = String, format = "ulid"),
        ("reaction_id" = String, format = "ulid"),
    ),
    responses(
        (status = 200, body = Reaction),
    ),
    security(
        ("access_key" = []),
    ),
)]
#[tracing::instrument(skip(data, _access))]
async fn get_reaction(
    data: Data<State>,
    _access: Access,
    extract::Path(id): extract::Path<Ulid>,
) -> Result<Json<Reaction>> {
    let reaction = reaction::Entity::find_by_id(id)
        .one(&*data.db)
        .await
        .context_internal_server_error("failed to query database")?
        .context_not_found("reaction not found")?;
    let user = if reaction.user_id.is_some() {
        Some(
            reaction
                .find_related(user::Entity)
                .one(&*data.db)
                .await
                .context_internal_server_error("failed to query database")?
                .context_internal_server_error("user not found")?,
        )
    } else {
        None
    };
    Ok(Json(Reaction::from_model(reaction, user)?))
}
