use activitypub_federation::{
    config::Data, fetch::webfinger::Webfinger, protocol::context::WithContext, traits::Object,
};
use axum::{extract, routing, Json, Router};
use derivative::Derivative;
use reqwest::header;
use serde::Deserialize;
use url::Url;
use utoipa::IntoParams;

use crate::{
    ap::{person::Person, Object as ApObject},
    config::CONFIG,
    dto::{self, User},
    entity::{post, user},
    error::{Context, Result},
    state::State,
};

use super::auth::Access;

pub(super) fn create_router() -> Router {
    Router::new()
        .route("/user", routing::get(get_user))
        .route("/link", routing::get(get_link))
}

#[derive(Debug, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
struct GetResolveUserQuery {
    handle: String,
    host: String,
}

#[utoipa::path(
    get,
    path = "/api/resolve/user",
    params(GetResolveUserQuery),
    responses(
        (status = 200, body = User),
    ),
    security(
        ("access_key" = []),
    ),
)]
#[tracing::instrument(skip(data, _access))]
async fn get_user(
    data: Data<State>,
    _access: Access,
    extract::Query(query): extract::Query<GetResolveUserQuery>,
) -> Result<Json<User>> {
    let url = if CONFIG.debug {
        format!(
            "http://{}/.well-known/webfinger?resource=acct:{}@{}",
            query.host, query.handle, query.host
        )
    } else {
        format!(
            "https://{}/.well-known/webfinger?resource=acct:{}@{}",
            query.host, query.handle, query.host
        )
    };
    let resp = data
        .http_client
        .get(url)
        .send()
        .await
        .context_internal_server_error("failed to request HTTP")?
        .error_for_status()
        .context_internal_server_error("target server returned error")?
        .json::<Webfinger>()
        .await
        .context_internal_server_error("failed to parse webfinger response")?;
    let activity_url = resp
        .links
        .into_iter()
        .find(|link| link.kind.as_deref() == Some("application/activity+json"))
        .and_then(|link| link.href)
        .context_internal_server_error("failed to find webfinger link")?;
    let person = data
        .http_client
        .get(activity_url)
        .header(header::ACCEPT, "application/activity+json")
        .send()
        .await
        .context_internal_server_error("failed to request HTTP")?
        .error_for_status()
        .context_internal_server_error("target server returned error")?
        .json::<WithContext<Person>>()
        .await
        .context_internal_server_error("failed to parse ActivityPub response")?;
    let user = user::Model::from_json(person.inner().clone(), &data).await?;
    Ok(Json(User::from_model(user)?))
}

#[derive(Derivative, Deserialize, IntoParams)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
struct GetResolveLinkQuery {
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    #[param(value_type = String, format = "url")]
    link: Url,
}

#[utoipa::path(
    get,
    path = "/api/resolve/link",
    params(GetResolveLinkQuery),
    responses(
        (status = 200, body = Object),
    ),
    security(
        ("access_key" = []),
    ),
)]
#[tracing::instrument(skip(data, _access))]
async fn get_link(
    data: Data<State>,
    _access: Access,
    extract::Query(query): extract::Query<GetResolveLinkQuery>,
) -> Result<Json<dto::Object>> {
    let object = data
        .http_client
        .get(query.link)
        .header(header::ACCEPT, "application/activity+json")
        .send()
        .await
        .context_internal_server_error("failed to request HTTP")?
        .error_for_status()
        .context_internal_server_error("target server returned error")?
        .json::<WithContext<ApObject>>()
        .await
        .context_internal_server_error("failed to parse ActivityPub response")?;
    let object = object.inner().clone();
    let dto = match object {
        ApObject::Note(note) => {
            let model = post::Model::from_json(*note, &data).await?;
            dto::Object::Post(Box::new(dto::Post::from_model(model, &*data.db).await?))
        }
        ApObject::Person(person) => {
            let model = user::Model::from_json(*person, &data).await?;
            dto::Object::User(Box::new(dto::User::from_model(model)?))
        }
    };
    Ok(Json(dto))
}
