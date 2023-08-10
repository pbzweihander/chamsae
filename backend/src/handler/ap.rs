use activitypub_federation::{
    axum::{
        inbox::{receive_activity, ActivityData},
        json::FederationJson,
    },
    config::Data,
    protocol::{context::WithContext, public_key::PublicKey},
};
// use axum::{routing, Router};

use crate::{
    ap::{Person, PersonAcceptedActivity},
    config::CONFIG,
    entity::user,
    error::Result,
};

use super::AppState;

// pub(super) fn create_router() -> Router {
//     Router::new()
//         .route("/user", routing::get(get_user))
//         .route("/inbox", routing::post(post_inbox))
// }

pub(super) async fn get_user() -> FederationJson<WithContext<Person>> {
    let id = CONFIG.user_id.clone().unwrap();
    let user = Person {
        ty: Default::default(),
        id: id.clone().into(),
        preferred_username: CONFIG.user_handle.clone(),
        name: None,
        inbox: CONFIG.inbox_url.clone().unwrap(),
        public_key: PublicKey {
            id: format!("{}#main-key", id),
            owner: id,
            public_key_pem: CONFIG.user_public_key.clone(),
        },
    };
    FederationJson(WithContext::new_default(user))
}

pub(super) async fn post_inbox(data: Data<AppState>, activity_data: ActivityData) -> Result<()> {
    receive_activity::<WithContext<PersonAcceptedActivity>, user::Model, AppState>(
        activity_data,
        &data,
    )
    .await
}
