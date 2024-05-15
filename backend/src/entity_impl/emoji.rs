use activitypub_federation::{
    config::Data, protocol::verification::verify_domains_match, traits::Object,
};
use async_trait::async_trait;
use sea_orm::{EntityTrait, ModelTrait};
use url::Url;

use crate::{
    ap::tag::{Emoji, EmojiIcon},
    config::CONFIG,
    entity::{emoji, local_file},
    error::{Context, Error},
    format_err,
    state::State,
};

impl emoji::Model {
    pub fn ap_id(&self) -> Result<Url, Error> {
        Url::parse(&format!(
            "https://{}/emoji/{}",
            CONFIG.public_domain, self.name
        ))
        .context_internal_server_error("failed to construct follow URL ID")
    }

    pub fn parse_ap_id(url: &str) -> Option<String> {
        url.strip_prefix(&format!("https://{}/emoji/", CONFIG.public_domain))
            .map(str::to_string)
    }
}

#[async_trait]
impl Object for emoji::Model {
    type DataType = State;
    type Kind = Emoji;
    type Error = Error;

    #[tracing::instrument(skip(data))]
    async fn read_from_id(
        object_id: Url,
        data: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        if let Some(id) = emoji::Model::parse_ap_id(object_id.as_str()) {
            emoji::Entity::find_by_id(id)
                .one(&*data.db)
                .await
                .context_internal_server_error("failed to query database")
        } else {
            Ok(None)
        }
    }

    #[tracing::instrument(skip(data))]
    async fn into_json(self, data: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        let file = self
            .find_related(local_file::Entity)
            .one(&*data.db)
            .await
            .context_internal_server_error("failed to query database")?
            .context_internal_server_error("file not found")?;

        Ok(Self::Kind {
            ty: Default::default(),
            id: self.ap_id()?,
            name: self.name,
            icon: EmojiIcon {
                ty: Default::default(),
                media_type: file
                    .media_type
                    .parse()
                    .context_internal_server_error("malformed file media type")?,
                url: file
                    .url
                    .parse()
                    .context_internal_server_error("malformed file URL")?,
            },
        })
    }

    async fn verify(
        json: &Self::Kind,
        expected_domain: &Url,
        _data: &Data<Self::DataType>,
    ) -> Result<(), Self::Error> {
        verify_domains_match(&json.id, expected_domain)
            .context_bad_request("failed to verify domain")
    }

    async fn from_json(
        _json: Self::Kind,
        _data: &Data<Self::DataType>,
    ) -> Result<Self, Self::Error> {
        Err(format_err!(NOT_IMPLEMENTED, "unimplemented"))
    }
}
