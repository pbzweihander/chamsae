use activitypub_federation::{
    axum::inbox::{receive_activity, ActivityData},
    config::Data,
    protocol::context::WithContext,
};

use crate::{ap::Activity, error::Result};

use super::State;

pub mod follow;
pub mod like;
pub mod note;
pub mod person;

#[tracing::instrument(skip(data, activity_data))]
pub(super) async fn post_inbox(data: Data<State>, activity_data: ActivityData) -> Result<()> {
    receive_activity::<WithContext<Activity>, crate::entity::user::Model, State>(
        activity_data,
        &data,
    )
    .await
}
