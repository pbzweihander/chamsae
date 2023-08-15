use activitypub_federation::{
    activity_queue::send_activity, config::Data, protocol::context::WithContext,
    traits::ActivityHandler,
};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use url::Url;

use crate::{
    config::CONFIG,
    error::{Context, Error},
    state::State,
};

pub mod announce;
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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum NoteOrAnnounce {
    Note(self::note::Note),
    Announce(self::announce::Announce),
}

impl NoteOrAnnounce {
    #[tracing::instrument(skip(data))]
    pub async fn send(self, data: &Data<State>, inboxes: Vec<Url>) -> Result<(), Error> {
        let me = self::person::LocalPerson::get(&*data.db).await?;
        match self {
            Self::Note(note) => {
                let create_note = self::note::CreateNote::new(note)?;
                let with_context = WithContext::new_default(create_note);
                send_activity(with_context, &me, inboxes, data).await
            }
            Self::Announce(announce) => {
                let with_context = WithContext::new_default(announce);
                send_activity(with_context, &me, inboxes, data).await
            }
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum Activity {
    AcceptFollow(self::follow::FollowAccept),
    Announce(self::announce::Announce),
    CreateFollow(self::follow::Follow),
    CreateNote(self::note::CreateNote),
    Delete(self::delete::Delete),
    Flag(self::flag::Flag),
    Like(self::like::Like),
    RejectFollow(self::follow::FollowReject),
    UndoFollow(self::undo::Undo<self::follow::Follow>),
    UndoLike(self::undo::Undo<self::like::Like>),
    UpdatePerson(Box<self::person::PersonUpdate>),
    /// Fallback
    Other(self::other_activity::OtherActivity),
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Object {
    Note(Box<self::note::Note>),
    Person(Box<self::person::Person>),
    Announce(Box<self::announce::Announce>),
}
