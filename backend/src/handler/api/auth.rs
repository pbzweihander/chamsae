use async_trait::async_trait;
use axum::{
    extract::{self, rejection::TypedHeaderRejectionReason, FromRequestParts},
    headers,
    http::{header, request::Parts, HeaderMap},
    response::Redirect,
    routing, Json, RequestPartsExt, Router, TypedHeader,
};
use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait};
use serde::Deserialize;
use time::OffsetDateTime;
use ulid::Ulid;

use crate::{
    config::CONFIG,
    entity::access_key,
    error::{Context, Error, Result},
    format_err,
    handler::AppState,
};

pub struct Access {
    pub key: access_key::Model,
}

#[async_trait]
impl FromRequestParts<AppState> for Access {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self> {
        let cookies = parts
            .extract::<TypedHeader<headers::Cookie>>()
            .await
            .map_err(|e| match *e.name() {
                header::COOKIE => match e.reason() {
                    TypedHeaderRejectionReason::Missing => {
                        format_err!(UNAUTHORIZED, "user not authorized")
                    }
                    _ => format_err!(INTERNAL_SERVER_ERROR, "failed to authorize"),
                },
                _ => format_err!(INTERNAL_SERVER_ERROR, "failed to authorize"),
            })?;
        let access_key_id = cookies
            .get("ACCESS_KEY")
            .ok_or(format_err!(UNAUTHORIZED, "user not authorized"))?;

        let access_key = access_key::Entity::find_by_id(access_key_id)
            .one(&*state.db)
            .await
            .context_internal_server_error("failed to request database")?
            .ok_or_else(|| format_err!(UNAUTHORIZED, "user not authorized"))?;

        let mut access_key_activemodel: access_key::ActiveModel = access_key.into();
        access_key_activemodel.last_used_at = ActiveValue::Set(Some(OffsetDateTime::now_utc()));
        let access_key = access_key_activemodel
            .update(&*state.db)
            .await
            .context_internal_server_error("failed to update database")?;

        Ok(Access { key: access_key })
    }
}

pub(super) fn create_router() -> Router<AppState> {
    Router::new()
        .route("/login", routing::post(post_login))
        .route("/check", routing::get(get_check))
}

#[derive(Deserialize)]
struct PostLoginQuery {
    redirect_to: Option<String>,
}

#[derive(Deserialize)]
struct PostLoginReq {
    id: String,
    password: String,
    hostname: String,
}

async fn post_login(
    extract::State(state): extract::State<AppState>,
    extract::Query(query): extract::Query<PostLoginQuery>,
    Json(req): Json<PostLoginReq>,
) -> Result<(HeaderMap, Redirect)> {
    if CONFIG.user_handle == req.id {
        if bcrypt::verify(&req.password, &CONFIG.user_password_bcrypt)
            .context_bad_request("failed to authenticate")?
        {
            let access_key_activemodel = access_key::ActiveModel {
                id: ActiveValue::Set(Ulid::new().to_string()),
                name: ActiveValue::Set(req.hostname),
                created_at: ActiveValue::Set(OffsetDateTime::now_utc()),
                last_used_at: ActiveValue::NotSet,
            };
            let access_key = access_key_activemodel
                .insert(&*state.db)
                .await
                .context_internal_server_error("failed to insert to database")?;

            let mut header_map = HeaderMap::new();
            header_map.insert(
                header::COOKIE,
                format!("ACCESS_KEY={}; SameSite=Lax; Path=/", access_key.id)
                    .parse()
                    .context_internal_server_error("failed to generate header value")?,
            );

            let redirect_to = query.redirect_to.as_deref().unwrap_or("/");

            return Ok((header_map, Redirect::to(redirect_to)));
        }
    }

    Err(format_err!(BAD_REQUEST, "failed to authenticate"))
}

async fn get_check(_access: Access) {
    // noop
}
