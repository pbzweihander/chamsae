use activitypub_federation::config::Data;
use axum::{extract, routing, Json, Router};
use sea_orm::{
    ColumnTrait, EntityTrait, ModelTrait, QueryFilter, QueryOrder, QuerySelect, TransactionTrait,
};
use ulid::Ulid;

use crate::{
    ap::follow::FollowReject,
    dto::{IdPaginationQuery, User},
    entity::{follower, user},
    error::{Context, Result},
    state::State,
};

use super::auth::Access;

pub(super) fn create_router() -> Router {
    Router::new()
        .route("/", routing::get(get_followers))
        .route("/:id", routing::delete(delete_follower))
}

#[utoipa::path(
    get,
    path = "/api/follower",
    params(IdPaginationQuery),
    responses(
        (status = 200, body = Vec<User>),
    ),
    security(
        ("access_key" = []),
    ),
)]
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

#[utoipa::path(
    delete,
    path = "/api/follower/{id}",
    params(
        ("id" = String, format = "ulid"),
    ),
    responses(
        (status = 200),
    ),
    security(
        ("access_key" = []),
    ),
)]
#[tracing::instrument(skip(data, _access))]
async fn delete_follower(
    data: Data<State>,
    extract::Path(id): extract::Path<Ulid>,
    _access: Access,
) -> Result<()> {
    let tx = data
        .db
        .begin()
        .await
        .context_internal_server_error("failed to begin database transaction")?;

    let (follower, user) = follower::Entity::find_by_id(id)
        .find_also_related(user::Entity)
        .one(&tx)
        .await
        .context_internal_server_error("failed to query database")?
        .context_bad_request("follower not found")?;
    let user = user.context_internal_server_error("user not found")?;

    follower
        .delete(&tx)
        .await
        .context_internal_server_error("failed to delete from database")?;

    tx.commit()
        .await
        .context_internal_server_error("failed to commit database transaction")?;

    let reject = FollowReject::new(
        user.id.into(),
        user.uri
            .parse()
            .context_internal_server_error("malformed user URI")?,
    )?;
    reject
        .send(
            &data,
            user.inbox
                .parse()
                .context_internal_server_error("malformed user inbox URL")?,
        )
        .await?;

    Ok(())
}
