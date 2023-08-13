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
pub mod flag;
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
    AcceptFollow(self::follow::FollowAccept),
    CreateFollow(self::follow::Follow),
    CreateNote(self::note::CreateNote),
    Delete(self::delete::Delete),
    Flag(self::flag::Flag),
    Like(self::like::Like),
    RejectFollow(self::follow::FollowReject),
    UndoFollow(self::undo::Undo<self::follow::Follow, follower::Model>),
    UndoLike(self::undo::Undo<self::like::Like, reaction::Model>),
    UpdatePerson(Box<self::person::PersonUpdate>),
    /// Fallback
    Other(self::other_activity::OtherActivity),
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Object {
    Note(Box<self::note::Note>),
    Person(Box<self::person::Person>),
}
