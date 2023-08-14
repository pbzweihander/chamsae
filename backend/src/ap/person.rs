use activitypub_federation::{
    activity_queue::send_activity,
    config::Data,
    fetch::object_id::ObjectId,
    kinds::{activity::UpdateType, object::ImageType, public},
    protocol::{context::WithContext, public_key::PublicKey, verification::verify_domains_match},
    traits::{ActivityHandler, Actor, Object},
};
use async_trait::async_trait;
use derivative::Derivative;
use once_cell::sync::Lazy;
use sea_orm::{EntityTrait, QuerySelect, TransactionTrait};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    config::CONFIG,
    entity::{local_file, setting, user},
    error::{Context, Error},
    format_err,
    queue::Notification,
    state::State,
    util::get_follower_inboxes,
};

use super::generate_object_id;

#[derive(Clone, Derivative, Deserialize, Serialize)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct PersonImage {
    #[serde(rename = "type")]
    pub ty: ImageType,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub url: Url,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum ActorType {
    Person,
    Service,
    Application,
}

impl std::fmt::Display for ActorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Person => write!(f, "Person"),
            Self::Service => write!(f, "Service"),
            Self::Application => write!(f, "Application"),
        }
    }
}

#[derive(Clone, Derivative, Deserialize, Serialize)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    #[serde(rename = "type")]
    pub ty: ActorType,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub id: ObjectId<user::Model>,
    pub preferred_username: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub icon: Option<PersonImage>,
    #[serde(default)]
    pub image: Option<PersonImage>,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub inbox: Url,
    #[derivative(Debug(format_with = "crate::fmt::debug_format_option_display"))]
    #[serde(default)]
    pub shared_inbox: Option<Url>,
    #[serde(default)]
    pub manually_approves_followers: bool,
    pub public_key: PublicKey,
}

#[derive(Debug)]
pub struct LocalPerson(pub setting::Model);

impl LocalPerson {
    pub async fn get(db: &impl TransactionTrait) -> Result<Self, Error> {
        Ok(Self(setting::Model::get(db).await?))
    }

    pub fn followers() -> Result<Url, Error> {
        Url::parse(&format!("{}/followers", Self::id()))
            .context_internal_server_error("failed to construct followers URL")
    }

    pub fn id() -> Url {
        static ID: Lazy<Url> = Lazy::new(|| {
            Url::parse(&format!("https://{}/ap/person", CONFIG.domain))
                .expect("failed to construct ID URL")
        });
        ID.clone()
    }

    pub fn inbox() -> Url {
        static INBOX: Lazy<Url> = Lazy::new(|| {
            Url::parse(&format!("https://{}/ap/inbox", CONFIG.domain))
                .expect("failed to construct inbox URL")
        });
        INBOX.clone()
    }
}

#[async_trait]
impl Object for LocalPerson {
    type DataType = State;
    type Kind = Person;
    type Error = Error;

    #[tracing::instrument(skip(data))]
    async fn read_from_id(
        object_id: Url,
        data: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        let this = Self::get(&*data.db).await?;
        if object_id == this.id() {
            Ok(Some(this))
        } else {
            Ok(None)
        }
    }

    #[tracing::instrument(skip(data))]
    async fn into_json(self, data: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        let id = self.id();

        let setting = setting::Model::get(&*data.db).await?;

        let avatar_url = if let Some(file_id) = setting.avatar_file_id {
            let url = local_file::Entity::find_by_id(file_id)
                .select_only()
                .column(local_file::Column::Url)
                .into_tuple::<String>()
                .one(&*data.db)
                .await
                .context_internal_server_error("failed to query database")?
                .context_internal_server_error("file not found")?;
            Some(Url::parse(&url).context_internal_server_error("malformed file URL")?)
        } else {
            None
        };
        let banner_url = if let Some(file_id) = setting.banner_file_id {
            let url = local_file::Entity::find_by_id(file_id)
                .select_only()
                .column(local_file::Column::Url)
                .into_tuple::<String>()
                .one(&*data.db)
                .await
                .context_internal_server_error("failed to query database")?
                .context_internal_server_error("file not found")?;
            Some(Url::parse(&url).context_internal_server_error("malformed file URL")?)
        } else {
            None
        };

        Ok(Self::Kind {
            ty: ActorType::Person,
            id: id.clone().into(),
            preferred_username: CONFIG.user_handle.clone(),
            name: setting.user_name,
            summary: setting.user_description,
            icon: avatar_url.map(|url| PersonImage {
                ty: Default::default(),
                url,
            }),
            image: banner_url.map(|url| PersonImage {
                ty: Default::default(),
                url,
            }),
            inbox: self.inbox(),
            shared_inbox: Some(self.inbox()),
            public_key: PublicKey {
                id: format!("{}#main-key", id),
                owner: id,
                public_key_pem: self.public_key_pem().to_string(),
            },
            manually_approves_followers: false,
        })
    }

    #[tracing::instrument(skip(_data))]
    async fn verify(
        json: &Self::Kind,
        expected_domain: &Url,
        _data: &Data<Self::DataType>,
    ) -> Result<(), Self::Error> {
        verify_domains_match(json.id.inner(), expected_domain)
            .context_bad_request("failed to verify domain")
    }

    #[tracing::instrument(skip(_data))]
    async fn from_json(
        _json: Self::Kind,
        _data: &Data<Self::DataType>,
    ) -> Result<Self, Self::Error> {
        Err(format_err!(NOT_IMPLEMENTED, "unimplemented"))
    }
}

impl Actor for LocalPerson {
    fn id(&self) -> Url {
        Self::id()
    }

    fn public_key_pem(&self) -> &str {
        &self.0.user_public_key
    }

    fn private_key_pem(&self) -> Option<String> {
        Some(self.0.user_private_key.clone())
    }

    fn inbox(&self) -> Url {
        Self::inbox()
    }
}

#[derive(Derivative, Deserialize, Serialize)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct PersonUpdate {
    #[serde(rename = "type")]
    pub ty: UpdateType,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub id: Url,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub actor: Url,
    #[derivative(Debug(format_with = "crate::fmt::debug_format_vec_display"))]
    #[serde(default)]
    pub to: Vec<Url>,
    pub object: Person,
}

impl PersonUpdate {
    pub async fn new_self(data: &Data<State>) -> Result<Self, Error> {
        let me = LocalPerson::get(&*data.db).await?;
        let me = me.into_json(data).await?;
        Ok(Self {
            ty: Default::default(),
            id: generate_object_id()?,
            actor: me.id.clone().into_inner(),
            to: vec![public()],
            object: me,
        })
    }

    pub async fn send(self, data: &Data<State>) -> Result<(), Error> {
        let me = LocalPerson::get(&*data.db).await?;
        let inboxes = get_follower_inboxes(&*data.db).await?;
        let with_context = WithContext::new_default(self);
        send_activity(with_context, &me, inboxes, data).await
    }
}

#[async_trait]
impl ActivityHandler for PersonUpdate {
    type DataType = State;
    type Error = Error;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        &self.actor
    }

    #[tracing::instrument(skip(_data))]
    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        verify_domains_match(&self.id, self.object.id.inner())
            .context_bad_request("failed to verify domain")
    }

    #[tracing::instrument(skip(data))]
    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let user = user::Model::from_json(self.object, data).await?;
        let notification = Notification::UpdateUser {
            user_id: user.id.into(),
        };
        notification.send(&mut data.redis()).await?;
        Ok(())
    }
}
