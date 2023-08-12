use activitypub_federation::{
    axum::inbox::{receive_activity, ActivityData},
    config::Data,
    protocol::context::WithContext,
};
use axum::Router;

use crate::{ap::Activity, error::Result};

use super::State;

mod follow;
mod like;
mod note;
mod person;

pub(super) fn create_router() -> Router {
    let follow = self::follow::create_router();
    let like = self::like::create_router();
    let note = self::note::create_router();
    let person = self::person::create_router();

    Router::new()
        .nest("/follow", follow)
        .nest("/like", like)
        .nest("/note", note)
        .nest("/person", person)
}

#[tracing::instrument(skip(data, activity_data))]
pub(super) async fn post_inbox(data: Data<State>, activity_data: ActivityData) -> Result<()> {
    receive_activity::<WithContext<Activity>, crate::entity::user::Model, State>(
        activity_data,
        &data,
    )
    .await
}
