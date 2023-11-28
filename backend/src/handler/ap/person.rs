use activitypub_federation::{
    axum::json::FederationJson, config::Data, protocol::context::WithContext, traits::Object,
};
use axum::{
    http::{header, HeaderMap},
    routing, Router,
};

use crate::{
    ap::person::{LocalPerson, Person},
    error::Result,
    handler::frontend::RespOrFrontend,
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
    if headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.starts_with("application/activity+json"))
        .unwrap_or_default()
    {
        let me = LocalPerson::get(&*data.db).await?;
        let me = me.into_json(&data).await?;
        Ok(RespOrFrontend::Resp(FederationJson(
            WithContext::new_default(me),
        )))
    } else {
        Ok(RespOrFrontend::Frontend)
    }
}
