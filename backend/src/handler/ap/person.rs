use activitypub_federation::{
    axum::json::FederationJson, config::Data, protocol::context::WithContext, traits::Object,
};
use axum::{routing, Router};

use crate::{
    ap::person::{LocalPerson, Person},
    error::Result,
    state::State,
};

pub(super) fn create_router() -> Router {
    Router::new().route("/", routing::get(get_person))
}

#[tracing::instrument(skip(data))]
async fn get_person(data: Data<State>) -> Result<FederationJson<WithContext<Person>>> {
    let this = LocalPerson.into_json(&data).await?;
    Ok(FederationJson(WithContext::new_default(this)))
}
