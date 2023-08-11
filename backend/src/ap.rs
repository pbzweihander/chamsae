use activitypub_federation::{config::Data, traits::ActivityHandler};
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use crate::{
    config::CONFIG,
    entity::follower,
    error::{Context, Error},
};

pub mod delete;
pub mod follow;
pub mod note;
pub mod other_activity;
pub mod person;
pub mod undo;

pub fn generate_object_id() -> Result<Url, Error> {
    Url::parse(&format!(
        "https://{}/ap/object/{}",
        CONFIG.domain,
        Uuid::new_v4()
    ))
    .context_internal_server_error("failed to construct object URL")
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum Activity {
    CreateNote(self::note::CreateNote),
    CreateFollow(self::follow::Follow),
    Accept(self::follow::FollowAccept),
    UndoFollow(self::undo::Undo<self::follow::Follow, follower::Model>),
    Delete(self::delete::Delete),
    Reject(self::follow::FollowReject),
    /// Fallback
    Other(self::other_activity::OtherActivity),
}
