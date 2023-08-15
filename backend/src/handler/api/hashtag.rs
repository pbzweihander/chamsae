use activitypub_federation::config::Data;
use axum::{extract, routing, Json, Router};
use futures_util::{stream::FuturesOrdered, TryStreamExt};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};

use crate::{
    dto::{IdPaginationQuery, Post},
    entity::{hashtag, post},
    error::{Context, Result},
    state::State,
};

use super::auth::Access;

pub(super) fn create_router() -> Router {
    Router::new().route("/:name", routing::get(get_hashtag_posts))
}

#[utoipa::path(
    get,
    path = "/api/hashtag/{name}",
    params(
        IdPaginationQuery,
        ("name" = String,),
    ),
    responses(
        (status = 200, body = Vec<Post>),
    ),
    security(
        ("access_key" = []),
    ),
)]
#[tracing::instrument(skip(data, _access))]
async fn get_hashtag_posts(
    data: Data<State>,
    _access: Access,
    extract::Path(name): extract::Path<String>,
    extract::Query(query): extract::Query<IdPaginationQuery>,
) -> Result<Json<Vec<Post>>> {
    let pagination_query = hashtag::Entity::find().find_also_related(post::Entity);
    let pagination_query = if let Some(after) = query.after {
        pagination_query.filter(post::Column::Id.lt(uuid::Uuid::from(after)))
    } else {
        pagination_query
    };
    let posts = pagination_query
        .filter(hashtag::Column::Name.eq(name))
        .order_by_desc(post::Column::Id)
        .limit(query.size)
        .all(&*data.db)
        .await
        .context_internal_server_error("failed to query database")?;
    let posts = posts
        .into_iter()
        .filter_map(|(_, post)| post)
        .map(|post| Post::from_model(post, &*data.db))
        .collect::<FuturesOrdered<_>>()
        .try_collect()
        .await?;
    Ok(Json(posts))
}
