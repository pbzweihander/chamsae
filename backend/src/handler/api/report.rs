use activitypub_federation::config::Data;
use axum::{extract, routing, Json, Router};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use ulid::Ulid;
use url::Url;

use crate::{
    ap::flag::Flag,
    dto::{CreateReport, IdPaginationQuery, Report},
    entity::{report, user},
    error::{Context, Result},
    state::State,
};

use super::auth::Access;

pub(super) fn create_router() -> Router {
    Router::new()
        .route("/", routing::get(get_reports).post(post_report))
        .route("/:id", routing::get(get_report))
}

#[utoipa::path(
    get,
    path = "/api/report",
    params(IdPaginationQuery),
    responses(
        (status = 200, body = Vec<Report>),
    ),
    security(
        ("access_key" = []),
    ),
)]
#[tracing::instrument(skip(data, _access))]
async fn get_reports(
    data: Data<State>,
    _access: Access,
    extract::Query(query): extract::Query<IdPaginationQuery>,
) -> Result<Json<Vec<Report>>> {
    let pagination_query = report::Entity::find().find_also_related(user::Entity);
    let pagination_query = if let Some(after) = query.after {
        pagination_query.filter(report::Column::Id.lt(uuid::Uuid::from(after)))
    } else {
        pagination_query
    };
    let reports = pagination_query
        .order_by_desc(report::Column::Id)
        .limit(100)
        .all(&*data.db)
        .await
        .context_internal_server_error("failed to query database")?;
    let reports = reports
        .into_iter()
        .filter_map(|(report, user)| user.map(|user| (report, user)))
        .filter_map(|(report, user)| Report::from_model(report, user).ok())
        .collect::<Vec<_>>();
    Ok(Json(reports))
}

#[utoipa::path(
    post,
    path = "/api/report",
    request_body = CreateReport,
    responses(
        (status = 200),
    ),
    security(
        ("access_key" = []),
    ),
)]
#[tracing::instrument(skip(data, _access))]
async fn post_report(
    data: Data<State>,
    _access: Access,
    Json(req): Json<CreateReport>,
) -> Result<()> {
    let (target_user_uri, inbox) = user::Entity::find_by_id(req.user_id)
        .select_only()
        .column(user::Column::Uri)
        .column(user::Column::Inbox)
        .into_tuple::<(String, String)>()
        .one(&*data.db)
        .await
        .context_internal_server_error("failed to query database")?
        .context_not_found("user not found")?;
    let target_user_uri =
        Url::parse(&target_user_uri).context_internal_server_error("malformed user URI")?;
    let inbox = Url::parse(&inbox).context_internal_server_error("malformed user inbox URL")?;
    let flag = Flag::new(target_user_uri, req.content)?;
    flag.send(&data, inbox).await?;
    Ok(())
}

#[utoipa::path(
    get,
    path = "/api/report/{id}",
    params(
        ("id" = String, format = "ulid"),
    ),
    responses(
        (status = 200, body = Report),
    ),
    security(
        ("access_key" = []),
    ),
)]
#[tracing::instrument(skip(data, _access))]
async fn get_report(
    data: Data<State>,
    _access: Access,
    extract::Path(id): extract::Path<Ulid>,
) -> Result<Json<Report>> {
    let (report, user) = report::Entity::find_by_id(id)
        .find_also_related(user::Entity)
        .one(&*data.db)
        .await
        .context_internal_server_error("failed to query database")?
        .context_not_found("report not found")?;
    let user = user.context_internal_server_error("user not found")?;
    Ok(Json(Report::from_model(report, user)?))
}
