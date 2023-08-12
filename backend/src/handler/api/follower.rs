use activitypub_federation::config::Data;
use axum::{extract, routing, Json, Router};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};

use crate::{
    dto::{IdPaginationQuery, User},
    entity::{follower, user},
    error::{Context, Result},
    state::State,
};

use super::auth::Access;

pub(super) fn create_router() -> Router {
    Router::new().route("/", routing::get(get_followers))
}

#[tracing::instrument(skip(data, _access))]
async fn get_followers(
    data: Data<State>,
    _access: Access,
    extract::Query(query): extract::Query<IdPaginationQuery>,
) -> Result<Json<Vec<User>>> {
    let pagination_query = follower::Entity::find().find_also_related(user::Entity);
    let pagination_query = if let Some(after) = query.after {
        pagination_query.filter(user::Column::Id.lt(uuid::Uuid::from(after)))
    } else {
        pagination_query
    };
    let followers = pagination_query
        .order_by_desc(user::Column::Id)
        .limit(100)
        .all(&*data.db)
        .await
        .context_internal_server_error("failed to query database")?;
    let followers = followers
        .into_iter()
        .filter_map(|(_, user)| user)
        .filter_map(|user| User::from_model(user).ok())
        .collect::<Vec<_>>();
    Ok(Json(followers))
}
