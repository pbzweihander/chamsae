use activitypub_federation::config::Data;
use async_trait::async_trait;
use axum::{
    extract::{rejection::TypedHeaderRejectionReason, FromRequestParts},
    headers,
    http::{header, request::Parts},
    routing, Json, RequestPartsExt, Router, TypedHeader,
};
use axum_client_ip::InsecureClientIp;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait, TransactionTrait};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use utoipa::ToSchema;

use crate::{
    entity::{access_key, setting},
    error::{Context, Error, Result},
    format_err,
    state::State,
};

pub struct Access {
    pub key: access_key::Model,
}

#[async_trait]
impl<S> FromRequestParts<S> for Access
where
    S: Clone + Send + Sync + 'static,
{
    type Rejection = Error;

    #[tracing::instrument(skip(parts, _state))]
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        let data = parts
            .extract::<Data<State>>()
            .await
            .map_err(|(code, message)| Error::new(code, message))?;
        let bearer = parts
            .extract::<TypedHeader<headers::Authorization<headers::authorization::Bearer>>>()
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

        let access_key_id =
            Ulid::from_string(bearer.token()).context_unauthorized("user not authorized")?;

        let tx = data
            .db
            .begin()
            .await
            .context_internal_server_error("failed to begin database transaction")?;

        let access_key = access_key::Entity::find_by_id(access_key_id)
            .one(&tx)
            .await
            .context_internal_server_error("failed to request database")?
            .context_unauthorized("user not authorized")?;

        let mut access_key_activemodel: access_key::ActiveModel = access_key.into();
        access_key_activemodel.last_used_at = ActiveValue::Set(Some(Utc::now().fixed_offset()));
        let access_key = access_key_activemodel
            .update(&tx)
            .await
            .context_internal_server_error("failed to update database")?;

        tx.commit()
            .await
            .context_internal_server_error("failed to commit database transaction")?;

        Ok(Access { key: access_key })
    }
}

pub(super) fn create_router() -> Router {
    Router::new()
        .route("/login", routing::post(post_login))
        .route("/check", routing::get(get_check))
}

#[derive(Deserialize, ToSchema)]
pub struct PostLoginReq {
    password: String,
}

#[derive(Serialize, ToSchema)]
pub struct PostLoginResp {
    #[schema(value_type = String, format = "ulid")]
    token: Ulid,
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = PostLoginReq,
    responses(
        (status = 200, body = PostLoginResp)
    )
)]
#[tracing::instrument(skip(data, req))]
async fn post_login(
    data: Data<State>,
    InsecureClientIp(client_ip): InsecureClientIp,
    Json(req): Json<PostLoginReq>,
) -> Result<Json<PostLoginResp>> {
    let setting = setting::Model::get(&*data.db).await?;
    if bcrypt::verify(&req.password, &setting.user_password_hash)
        .context_bad_request("failed to authenticate")?
    {
        let access_key_activemodel = access_key::ActiveModel {
            id: ActiveValue::Set(Ulid::new().into()),
            // TODO: Parse user agent for more expressive name?
            name: ActiveValue::Set(client_ip.to_string()),
            last_used_at: ActiveValue::NotSet,
        };
        let access_key = access_key_activemodel
            .insert(&*data.db)
            .await
            .context_internal_server_error("failed to insert to database")?;

        Ok(Json(PostLoginResp {
            token: access_key.id.into(),
        }))
    } else {
        Err(format_err!(BAD_REQUEST, "failed to authenticate"))
    }
}

#[utoipa::path(
    get,
    path = "/api/auth/check",
    responses(
        (status = 200),
    ),
    security(
        ("access_key" = []),
    ),
)]
async fn get_check(_access: Access) {
    // noop
}
