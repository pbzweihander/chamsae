use activitypub_federation::{
    activity_queue::send_activity,
    config::Data,
    fetch::object_id::ObjectId,
    kinds::{activity::DeleteType, object::TombstoneType},
    protocol::{context::WithContext, verification::verify_domains_match},
    traits::{ActivityHandler, Actor, Object},
};
use async_trait::async_trait;
use sea_orm::{EntityTrait, JoinType, QuerySelect, RelationTrait};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    entity::{follower, post, user},
    error::{Context, Error},
    state::State,
};

use super::{generate_object_id, person::LocalPerson};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tombstone {
    #[serde(rename = "type")]
    pub ty: TombstoneType,
    pub id: Url,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Delete {
    #[serde(rename = "type")]
    pub ty: DeleteType,
    pub id: Url,
    pub actor: Url,
    pub object: Tombstone,
}

impl Delete {
    pub fn new(id: Url) -> Result<Self, Error> {
        Ok(Self {
            ty: Default::default(),
            id: generate_object_id()?,
            actor: LocalPerson.id(),
            object: Tombstone {
                ty: Default::default(),
                id,
            },
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
        send_activity(with_context, &LocalPerson, inboxes, data).await
    }
}

#[async_trait]
impl ActivityHandler for Delete {
    type DataType = State;
    type Error = Error;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        &self.actor
    }

    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        verify_domains_match(&self.object.id, &self.id)
            .context_bad_request("failed to verify domain")
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let post_id: ObjectId<post::Model> = self.object.id.into();
        let res = post_id.dereference_local(data).await;
        match res {
            Ok(post) => {
                post.delete(data).await?;
                Ok(())
            }
            Err(error) => {
                if let Some(activitypub_federation::error::Error::NotFound) =
                    error
                        .inner
                        .downcast_ref::<activitypub_federation::error::Error>()
                {
                    Ok(())
                } else {
                    Err(error)
                }
            }
        }
    }
}
