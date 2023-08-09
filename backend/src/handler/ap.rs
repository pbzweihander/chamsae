use activitypub_federation::{
    axum::json::FederationJson,
    protocol::{context::WithContext, public_key::PublicKey},
};
use axum::{routing, Router};

use crate::{ap::Person, config::CONFIG};

pub(super) fn create_router() -> Router {
    Router::new().route("/user", routing::get(get_user))
}

async fn get_user() -> FederationJson<WithContext<Person>> {
    let id = CONFIG.user_id.clone().unwrap();
    let user = Person {
        id: id.clone().into(),
        ty: Default::default(),
        preferred_username: CONFIG.user_handle.clone(),
        name: CONFIG.user_handle.clone(),
        inbox: CONFIG.inbox_url.clone().unwrap(),
        public_key: PublicKey {
            id: format!("{}#main-key", id),
            owner: id,
            public_key_pem: CONFIG.user_public_key.clone(),
        },
    };
    FederationJson(WithContext::new_default(user))
}
