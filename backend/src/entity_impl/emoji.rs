use url::Url;

use crate::{
    config::CONFIG,
    entity::emoji,
    error::{Context, Result},
};

impl emoji::Model {
    pub fn ap_id(&self) -> Result<Url> {
        Url::parse(&format!("https://{}/ap/emoji/{}", CONFIG.domain, self.id))
            .context_internal_server_error("failed to construct follow URL ID")
    }
}
