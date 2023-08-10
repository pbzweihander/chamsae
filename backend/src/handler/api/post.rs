use axum::{extract, routing, Json, Router};
use chrono::{DateTime, FixedOffset, Utc};
use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait, PaginatorTrait, TransactionTrait};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::{
    entity::{post, sea_orm_active_enums, user},
    error::{Context, Result},
    format_err,
    handler::AppState,
};

use super::auth::Access;

pub(super) fn create_router() -> Router<AppState> {
    Router::new()
        .route("/", routing::post(post_post))
        .route("/:id", routing::get(get_post).delete(delete_post))
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
enum Visibility {
    Public,
    Home,
    Followers,
    DirectMessage,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PostPostReq {
    reply_id: Option<Ulid>,
    text: String,
    title: Option<String>,
    visibility: Visibility,
}

async fn post_post(
    extract::State(state): extract::State<AppState>,
    _access: Access,
    Json(req): Json<PostPostReq>,
) -> Result<()> {
    let tx = state
        .db
        .begin()
        .await
        .context_internal_server_error("failed to start database transaction")?;

    if let Some(reply_id) = &req.reply_id {
        let reply_post_count = post::Entity::find_by_id(reply_id.to_string())
            .count(&tx)
            .await
            .context_internal_server_error("failed to request database")?;
        if reply_post_count == 0 {
            return Err(format_err!(NOT_FOUND, "reply target post not found"));
        }
    }

    let post_activemodel = post::ActiveModel {
        id: ActiveValue::Set(Ulid::new().to_string()),
        created_at: ActiveValue::Set(Utc::now().fixed_offset()),
        reply_id: ActiveValue::Set(req.reply_id.as_ref().map(Ulid::to_string)),
        text: ActiveValue::Set(req.text),
        title: ActiveValue::Set(req.title),
        user_id: ActiveValue::Set(None),
        visibility: ActiveValue::Set(match req.visibility {
            Visibility::Public => sea_orm_active_enums::Visibility::Public,
            Visibility::Home => sea_orm_active_enums::Visibility::Home,
            Visibility::Followers => sea_orm_active_enums::Visibility::Followers,
            Visibility::DirectMessage => sea_orm_active_enums::Visibility::DirectMessage,
        }),
        uri: ActiveValue::Set(None),
    };
    post_activemodel
        .insert(&tx)
        .await
        .context_internal_server_error("failed to insert to database")?;

    tx.commit()
        .await
        .context_internal_server_error("failed to commmit database transaction")?;

    // TODO: broadcast via ActivityPub

    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetPostRespUser {
    handle: Ulid,
    host: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetPostResp {
    id: Ulid,
    created_at: DateTime<FixedOffset>,
    reply_id: Option<Ulid>,
    text: String,
    title: Option<String>,
    user: Option<GetPostRespUser>,
    visibility: Visibility,
    uri: Option<String>,
}

async fn get_post(
    extract::Path(id): extract::Path<Ulid>,
    extract::State(state): extract::State<AppState>,
    _access: Access,
) -> Result<Json<GetPostResp>> {
    let post = post::Entity::find_by_id(id.to_string())
        .one(&*state.db)
        .await
        .context_internal_server_error("failed to query database")?
        .ok_or_else(|| format_err!(NOT_FOUND, "post not found"))?;
    let user = if let Some(user_id) = &post.user_id {
        let user = user::Entity::find_by_id(user_id.to_string())
            .one(&*state.db)
            .await
            .context_internal_server_error("failed to query database")?
            .ok_or_else(|| format_err!(INTERNAL_SERVER_ERROR, "user not found"))?;
        Some(GetPostRespUser {
            handle: Ulid::from_string(&user.id).context_internal_server_error("malformed id")?,
            host: user.host,
        })
    } else {
        None
    };

    Ok(Json(GetPostResp {
        id: Ulid::from_string(&post.id).context_internal_server_error("malformed id")?,
        created_at: post.created_at,
        reply_id: post
            .reply_id
            .as_deref()
            .map(Ulid::from_string)
            .transpose()
            .context_internal_server_error("malformed id")?,
        text: post.text,
        title: post.title,
        user,
        visibility: match post.visibility {
            sea_orm_active_enums::Visibility::Public => Visibility::Public,
            sea_orm_active_enums::Visibility::Home => Visibility::Home,
            sea_orm_active_enums::Visibility::Followers => Visibility::Followers,
            sea_orm_active_enums::Visibility::DirectMessage => Visibility::DirectMessage,
        },
        uri: post.uri,
    }))
}

async fn delete_post(
    extract::Path(id): extract::Path<Ulid>,
    extract::State(state): extract::State<AppState>,
    _access: Access,
) -> Result<()> {
    let tx = state
        .db
        .begin()
        .await
        .context_internal_server_error("failed to begin database transaction")?;

    let post_count = post::Entity::find_by_id(id.to_string())
        .count(&tx)
        .await
        .context_internal_server_error("failed to query database")?;

    if post_count == 0 {
        return Err(format_err!(NOT_FOUND, "post not found"));
    }

    post::Entity::delete_by_id(id.to_string())
        .exec(&tx)
        .await
        .context_internal_server_error("failed to delete from database")?;

    tx.commit()
        .await
        .context_internal_server_error("failed to commit database transaction")?;

    // TODO: broadcast via ActivityPub

    Ok(())
}
