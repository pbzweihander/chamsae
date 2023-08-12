use activitypub_federation::{
    config::Data,
    protocol::verification::verify_domains_match,
    traits::{Actor, Object},
};
use async_trait::async_trait;
use sea_orm::{EntityTrait, ModelTrait, QuerySelect};
use url::Url;
use uuid::Uuid;

use crate::{
    ap::{follow::Follow, person::LocalPerson},
    config::CONFIG,
    entity::{follow, user},
    error::{Context, Error},
    format_err,
    state::State,
};

impl follow::Model {
    pub fn ap_id(&self) -> Result<Url, Error> {
        Url::parse(&format!(
            "https://{}/ap/follow/{}",
            CONFIG.domain, self.to_id
        ))
        .context_internal_server_error("failed to construct follow URL ID")
    }

    pub fn parse_ap_id(url: &str) -> Option<Uuid> {
        url.strip_prefix(&format!("https://{}/ap/follow/", CONFIG.domain))
            .and_then(|id| Uuid::parse_str(id).ok())
    }
}

#[async_trait]
impl Object for follow::Model {
    type DataType = State;
    type Kind = Follow;
    type Error = Error;

    #[tracing::instrument(skip(data))]
    async fn read_from_id(
        object_id: Url,
        data: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        let object_id = object_id.to_string();
        if let Some(id) = follow::Model::parse_ap_id(object_id.as_str()) {
            follow::Entity::find_by_id(id)
                .one(&*data.db)
                .await
                .context_internal_server_error("failed to query database")
        } else {
            Ok(None)
        }
    }

    #[tracing::instrument(skip(data))]
    async fn into_json(self, data: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        let to_user_id = self
            .find_related(user::Entity)
            .select_only()
            .column(user::Column::Uri)
            .into_tuple::<String>()
            .one(&*data.db)
            .await
            .context_internal_server_error("failed to query database")?
            .context_internal_server_error("failed to find target user")?;
        let to_user_id =
            Url::parse(&to_user_id).context_internal_server_error("malformed user URI")?;
        Ok(Self::Kind {
            ty: Default::default(),
            id: self.ap_id()?,
            actor: LocalPerson.id(),
            object: to_user_id,
        })
    }

    #[tracing::instrument(skip(_data))]
    async fn verify(
        json: &Self::Kind,
        expected_domain: &Url,
        _data: &Data<Self::DataType>,
    ) -> Result<(), Self::Error> {
        verify_domains_match(&json.id, expected_domain)
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
