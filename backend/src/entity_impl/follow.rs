use activitypub_federation::{
    config::Data, protocol::verification::verify_domains_match, traits::Object,
};
use async_trait::async_trait;
use sea_orm::{EntityTrait, QuerySelect};
use url::Url;

use crate::{
    ap::Follow,
    config::CONFIG,
    entity::{follow, user},
    error::{Context, Error},
    format_err,
    state::State,
};

impl follow::Model {
    pub fn ap_id(&self) -> Result<Url, Error> {
        Url::parse(&format!("https://{}/ap/follow/{}", CONFIG.domain, self.id))
            .context_internal_server_error("failed to construct follow URL ID")
    }

    pub fn parse_id_from_ap_id(url: &str) -> Option<String> {
        url.strip_prefix(&format!("https://{}/ap/follow/", CONFIG.domain))
            .map(str::to_string)
    }
}

#[async_trait]
impl Object for follow::Model {
    type DataType = State;
    type Kind = Follow;
    type Error = Error;

    async fn read_from_id(
        object_id: Url,
        data: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        let object_id = object_id.to_string();
        if let Some(id) = follow::Model::parse_id_from_ap_id(object_id.as_str()) {
            follow::Entity::find_by_id(id)
                .one(&*data.db)
                .await
                .context_internal_server_error("failed to query database")
        } else {
            Ok(None)
        }
    }

    async fn into_json(self, data: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        let to_user_id = user::Entity::find_by_id(&self.to_id)
            .select_only()
            .column(user::Column::Uri)
            .into_tuple::<String>()
            .one(&*data.db)
            .await
            .context_internal_server_error("failed to query database")?
            .ok_or_else(|| format_err!(INTERNAL_SERVER_ERROR, "failed to find target user"))?;
        let to_user_id =
            Url::parse(&to_user_id).context_internal_server_error("malformed user URI")?;
        Ok(Self::Kind {
            ty: Default::default(),
            id: self.ap_id()?.into(),
            actor: CONFIG.user_id.clone().unwrap(),
            object: to_user_id.into(),
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
