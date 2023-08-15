use activitypub_federation::config::Data;
use axum::{extract, routing, Json, Router};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use ulid::Ulid;

use crate::{
    dto::IdPaginationQuery,
    entity::notification,
    error::{Context, Error},
    queue::Notification,
    state::State,
};

use super::auth::Access;

pub(super) fn create_router() -> Router {
    Router::new()
        .route("/", routing::get(get_notifications))
        .route("/:id", routing::get(get_notification))
}

#[utoipa::path(
    get,
    path = "/api/notification",
    params(IdPaginationQuery),
    responses(
        (status = 200, body = Vec<Notification>),
    ),
    security(
        ("access_key" = []),
    ),
)]
#[tracing::instrument(skip(data, _access))]
async fn get_notifications(
    data: Data<State>,
    _access: Access,
    extract::Query(query): extract::Query<IdPaginationQuery>,
) -> Result<Json<Vec<Notification>>, Error> {
    let pagination_query = notification::Entity::find();
    let pagination_query = if let Some(after) = query.after {
        pagination_query.filter(notification::Column::Id.lt(uuid::Uuid::from(after)))
    } else {
        pagination_query
    };
    let notifications = pagination_query
        .order_by_desc(notification::Column::Id)
        .limit(query.size)
        .all(&*data.db)
        .await
        .context_internal_server_error("failed to query database")?;
    let notifications = notifications
        .into_iter()
        .filter_map(|notification| {
            Some(Notification {
                id: notification.id.into(),
                ty: serde_json::from_value(notification.payload).ok()?,
            })
        })
        .collect::<Vec<_>>();
    Ok(Json(notifications))
}

#[utoipa::path(
    get,
    path = "/api/notification/{id}",
    params(
        ("id" = String, format = "ulid"),
    ),
    responses(
        (status = 200, body = Notification),
    ),
    security(
        ("access_key" = []),
    ),
)]
#[tracing::instrument(skip(data, _access))]
async fn get_notification(
    data: Data<State>,
    _access: Access,
    extract::Path(id): extract::Path<Ulid>,
) -> Result<Json<Notification>, Error> {
    let notification = notification::Entity::find_by_id(id)
        .one(&*data.db)
        .await
        .context_internal_server_error("failed to query database")?
        .context_not_found("notification not found")?;
    let notification = Notification {
        id: notification.id.into(),
        ty: serde_json::from_value(notification.payload)
            .context_internal_server_error("malformed notification payload")?,
    };
    Ok(Json(notification))
}
