use activitypub_federation::{
    activity_queue::send_activity,
    config::Data,
    fetch::object_id::ObjectId,
    kinds::{
        activity::{AcceptType, CreateType, FollowType},
        actor::PersonType,
        link::MentionType,
        object::NoteType,
        public,
    },
    protocol::{
        context::WithContext, helpers::deserialize_one_or_many, public_key::PublicKey,
        verification::verify_domains_match,
    },
    traits::{ActivityHandler, Actor, Object},
};
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait, JoinType, QuerySelect, RelationTrait};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use url::Url;

use crate::{
    config::CONFIG,
    entity::{follow, follower, post, user},
    error::{Context, Error},
    format_err,
    state::State,
};

pub fn generate_object_id() -> Result<Url, Error> {
    Url::parse(&format!(
        "https://{}/ap/object/{}",
        CONFIG.domain,
        Ulid::new()
    ))
    .context_internal_server_error("failed to construct object URL")
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    #[serde(rename = "type")]
    pub ty: PersonType,
    pub id: ObjectId<user::Model>,
    pub preferred_username: String,
    pub name: Option<String>,
    pub inbox: Url,
    pub public_key: PublicKey,
}

#[derive(Debug)]
pub struct LocalUser;

#[async_trait]
impl Object for LocalUser {
    type DataType = State;
    type Kind = Person;
    type Error = Error;

    async fn read_from_id(
        object_id: Url,
        _data: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        if object_id == CONFIG.user_id.clone().unwrap() {
            Ok(Some(Self))
        } else {
            Ok(None)
        }
    }

    async fn into_json(self, _data: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        let id = CONFIG.user_id.clone().unwrap();
        Ok(Self::Kind {
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
        })
    }

    async fn verify(
        json: &Self::Kind,
        expected_domain: &Url,
        _data: &Data<Self::DataType>,
    ) -> Result<(), Self::Error> {
        verify_domains_match(json.id.inner(), expected_domain)
            .context_bad_request("failed to verify domain")
    }

    async fn from_json(
        _json: Self::Kind,
        _data: &Data<Self::DataType>,
    ) -> Result<Self, Self::Error> {
        Err(format_err!(NOT_IMPLEMENTED, "unimplemented"))
    }
}

impl Actor for LocalUser {
    fn id(&self) -> Url {
        CONFIG.user_id.clone().unwrap()
    }

    fn public_key_pem(&self) -> &str {
        &CONFIG.user_public_key
    }

    fn private_key_pem(&self) -> Option<String> {
        Some(CONFIG.user_private_key.clone())
    }

    fn inbox(&self) -> Url {
        CONFIG.inbox_url.clone().unwrap()
    }
}

#[derive(Deserialize, Serialize)]
pub struct Mention {
    #[serde(rename = "type")]
    pub ty: MentionType,
    pub href: Url,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    #[serde(rename = "type")]
    pub ty: NoteType,
    pub id: ObjectId<post::Model>,
    pub attributed_to: ObjectId<user::Model>,
    pub to: Vec<Url>,
    pub content: String,
    pub in_reply_to: Option<ObjectId<post::Model>>,
    pub tag: Vec<Mention>,
}

impl Note {
    pub fn into_create(self) -> Result<CreateNote, Error> {
        CreateNote::new(self)
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Follow {
    #[serde(rename = "type")]
    pub ty: FollowType,
    pub id: ObjectId<follow::Model>,
    pub actor: Url,
    pub object: ObjectId<user::Model>,
}

impl Follow {
    pub async fn send(self, data: &Data<State>) -> Result<(), Error> {
        let inbox = self.object.dereference(data).await?.inbox;
        let inbox = Url::parse(&inbox).context_internal_server_error("malformed user inbox URL")?;
        let with_context = WithContext::new_default(self);
        send_activity(with_context, &LocalUser, vec![inbox], data).await
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Follower {
    #[serde(rename = "type")]
    pub ty: FollowType,
    pub id: ObjectId<follower::Model>,
    pub actor: ObjectId<user::Model>,
    pub object: Url,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowAccept {
    #[serde(rename = "type")]
    pub ty: AcceptType,
    pub id: Url,
    pub actor: Url,
    pub object: Follow,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowerAccept {
    #[serde(rename = "type")]
    pub ty: AcceptType,
    pub id: Url,
    pub actor: Url,
    pub object: Follower,
}

impl FollowerAccept {
    pub async fn send(self, data: &Data<State>) -> Result<(), Error> {
        let inbox = self.object.actor.dereference(data).await?.inbox;
        let inbox = Url::parse(&inbox).context_internal_server_error("malformed user inbox URL")?;
        let with_context = WithContext::new_default(self);
        send_activity(with_context, &LocalUser, vec![inbox], data).await
    }
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum Activity {
    CreateNote(CreateNote),
    CreateFollower(Follower),
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateNote {
    #[serde(rename = "type")]
    pub ty: CreateType,
    pub id: Url,
    pub actor: ObjectId<user::Model>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub to: Vec<Url>,
    pub object: Note,
}

impl CreateNote {
    pub fn new(note: Note) -> Result<Self, Error> {
        Ok(Self {
            ty: Default::default(),
            id: generate_object_id()?,
            actor: note.attributed_to.clone(),
            to: vec![public()],
            object: note,
        })
    }

    pub async fn send(self, data: &Data<State>) -> Result<(), Error> {
        let inboxes = follower::Entity::find()
            .join(JoinType::InnerJoin, follower::Relation::User.def())
            .select_only()
            .column(user::Column::Inbox)
            .into_tuple::<String>()
            .all(&*data.db)
            .await
            .context_internal_server_error("failed to query database")?;
        let inboxes = inboxes
            .into_iter()
            .filter_map(|url| Url::parse(&url).ok())
            .collect::<Vec<_>>();
        let with_context = WithContext::new_default(self);
        send_activity(with_context, &LocalUser, inboxes, data).await
    }
}

#[async_trait]
impl ActivityHandler for CreateNote {
    type DataType = State;
    type Error = Error;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        post::Model::verify(&self.object, &self.id, data).await
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        post::Model::from_json(self.object, data).await?;
        Ok(())
    }
}

#[async_trait]
impl ActivityHandler for Follow {
    type DataType = State;
    type Error = Error;

    fn id(&self) -> &Url {
        self.id.inner()
    }

    fn actor(&self) -> &Url {
        &self.actor
    }

    async fn verify(&self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        follow::Model::verify(self, self.id.inner(), data).await
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        follow::Model::from_json(self, data).await?;
        Ok(())
    }
}

#[async_trait]
impl ActivityHandler for Follower {
    type DataType = State;
    type Error = Error;

    fn id(&self) -> &Url {
        self.id.inner()
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        follower::Model::verify(self, self.id.inner(), data).await
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        follower::Model::from_json(self.clone(), data).await?;
        let accept = FollowerAccept {
            ty: Default::default(),
            id: generate_object_id()?,
            actor: CONFIG.user_id.clone().unwrap(),
            object: self,
        };
        accept.send(data).await?;
        Ok(())
    }
}

#[async_trait]
impl ActivityHandler for FollowAccept {
    type DataType = State;
    type Error = Error;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        &self.actor
    }

    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        verify_domains_match(self.object.id.inner(), &self.id)
            .context_bad_request("failed to verify domain")
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let follow = self.object.id.dereference(data).await?;
        let mut follow_activemodel: follow::ActiveModel = follow.into();
        follow_activemodel.accepted = ActiveValue::Set(true);
        follow_activemodel
            .update(&*data.db)
            .await
            .context_internal_server_error("failed to update database")?;
        Ok(())
    }
}

#[async_trait]
impl ActivityHandler for FollowerAccept {
    type DataType = State;
    type Error = Error;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        &self.actor
    }

    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        verify_domains_match(self.object.id.inner(), &self.id)
            .context_bad_request("failed to verify domain")
    }

    async fn receive(self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        Err(format_err!(NOT_IMPLEMENTED, "unimplemented"))
    }
}
