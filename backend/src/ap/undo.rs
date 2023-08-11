use std::marker::PhantomData;

use activitypub_federation::{
    activity_queue::send_activity,
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::UndoType,
    protocol::{context::WithContext, verification::verify_domains_match},
    traits::{ActivityHandler, Actor, Object},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    error::{Context, Error},
    format_err,
    state::State,
};

use super::{generate_object_id, person::LocalPerson};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Undo<T, M> {
    #[serde(rename = "type")]
    pub ty: UndoType,
    pub id: Url,
    pub actor: Url,
    pub object: T,
    #[serde(skip)]
    pub m: PhantomData<M>,
}

impl<T, M, U> Undo<T, M>
where
    T: ActivityHandler + Serialize + Send + Sync + 'static,
    M: Object<DataType = State, Kind = U, Error = Error> + Send + Sync + 'static,
    for<'de> U: Deserialize<'de>,
{
    pub fn new(object: T) -> Result<Self, Error> {
        Ok(Self {
            ty: Default::default(),
            id: generate_object_id()?,
            actor: LocalPerson.id(),
            object,
            m: Default::default(),
        })
    }

    pub async fn send(self, data: &Data<State>, inboxes: Vec<Url>) -> Result<(), Error> {
        let with_context = WithContext::new_default(self);
        send_activity(with_context, &LocalPerson, inboxes, data).await
    }
}

#[async_trait]
impl<T, M, U> ActivityHandler for Undo<T, M>
where
    T: ActivityHandler + Send + Sync + 'static,
    M: Object<DataType = State, Kind = U, Error = Error> + Send + Sync + 'static,
    for<'de> U: Deserialize<'de>,
{
    type DataType = State;
    type Error = Error;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        &self.actor
    }

    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        verify_domains_match(self.object.id(), &self.id)
            .context_bad_request("failed to verify domain")
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let object_id: ObjectId<M> = self.object.id().clone().into();
        let res = object_id.dereference_local(data).await;
        match res {
            Ok(object) => {
                object.delete(data).await?;
                Ok(())
            }
            Err(error) => {
                if let Some(activitypub_federation::error::Error::NotFound) =
                    error
                        .inner
                        .downcast_ref::<activitypub_federation::error::Error>()
                {
                    Err(format_err!(NOT_FOUND, "not found"))
                } else {
                    Err(error)
                }
            }
        }
    }
}
