use activitypub_federation::{
    axum::json::FederationJson, config::Data, protocol::context::WithContext, traits::Object,
};
use axum::{
    http::{header, HeaderMap, StatusCode},
    routing, Router,
};

use crate::{
    ap::person::{LocalPerson, Person},
    error::Result,
    handler::frontend::{FrontendContext, RespOrFrontend},
    state::State,
};

pub fn create_router() -> Router {
    Router::new().route("/", routing::get(get_person))
}

#[tracing::instrument(skip(data))]
async fn get_person(
    data: Data<State>,
    headers: HeaderMap,
) -> Result<RespOrFrontend<FederationJson<WithContext<Person>>>> {
    let me = LocalPerson::get(&*data.db).await?;
    if headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.starts_with("application/activity+json"))
        .unwrap_or_default()
    {
        let me = me.into_json(&data).await?;
        Ok(RespOrFrontend::Resp(FederationJson(
            WithContext::new_default(me),
        )))
    } else {
        let name = me.display_name().to_string();
        let description = me.description().clone();
        let avatar_url = me
            .get_avatar_url(&*data.db)
            .await?
            .map(|url| url.to_string());

        let ctx = FrontendContext {
            title: Some(name.clone()),
            description: description.clone(),
            og_title: Some(name),
            og_type: None,
            og_description: description,
            og_image: avatar_url,
        };

        RespOrFrontend::frontend(StatusCode::OK, &*data.db, ctx).await
    }
}
