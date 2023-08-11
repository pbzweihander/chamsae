use activitypub_federation::{
    axum::{
        inbox::{receive_activity, ActivityData},
        json::FederationJson,
    },
    config::Data,
    protocol::context::WithContext,
    traits::Object,
};
// use axum::{routing, Router};

use crate::{
    ap::{
        person::{LocalPerson, Person},
        Activity,
    },
    entity::user,
    error::Result,
};

use super::State;

// pub(super) fn create_router() -> Router {
//     Router::new()
//         .route("/user", routing::get(get_user))
//         .route("/inbox", routing::post(post_inbox))
// }

#[tracing::instrument(skip(data))]
pub(super) async fn get_user(data: Data<State>) -> Result<FederationJson<WithContext<Person>>> {
    let user = LocalPerson.into_json(&data).await?;
    Ok(FederationJson(WithContext::new_default(user)))
}

#[tracing::instrument(skip(data, activity_data))]
pub(super) async fn post_inbox(data: Data<State>, activity_data: ActivityData) -> Result<()> {
    receive_activity::<WithContext<Activity>, user::Model, State>(activity_data, &data).await
}
