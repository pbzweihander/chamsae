use activitypub_federation::{config::Data, traits::ActivityHandler};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use url::Url;

use crate::{
    config::CONFIG,
    entity::{follower, reaction},
    error::{Context, Error},
};

pub mod delete;
pub mod follow;
pub mod like;
pub mod note;
pub mod other_activity;
pub mod person;
pub mod tag;
pub mod undo;

pub fn generate_object_id() -> Result<Url, Error> {
    Url::parse(&format!(
        "https://{}/ap/object/{}",
        CONFIG.domain,
        Ulid::new(),
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
    Like(self::like::Like),
    UndoLike(self::undo::Undo<self::like::Like, reaction::Model>),
    /// Fallback
    Other(self::other_activity::OtherActivity),
}
