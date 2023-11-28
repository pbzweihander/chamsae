use activitypub_federation::config::Data;
use askama::Template;
use axum::{http::StatusCode, response::IntoResponse};
use sea_orm::{ConnectionTrait, EntityTrait, QuerySelect};
use ulid::Ulid;

use crate::{
    entity::{local_file, setting},
    error::{Context, Result},
    state::State,
};

pub mod assets;

pub struct FrontendContext {
    pub title: Option<String>,
    pub description: Option<String>,
    pub og_type: Option<String>,
    pub og_title: Option<String>,
    pub og_description: Option<String>,
    pub og_image: Option<String>,
}

impl FrontendContext {
    pub async fn site_default(db: &impl ConnectionTrait) -> Result<Self> {
        let setting = setting::Entity::find_by_id(Ulid::nil())
            .one(db)
            .await
            .context_internal_server_error("failed to query database")?;

        let (title, description) = if let Some(setting) = setting {
            (Some(setting.instance_name), setting.instance_description)
        } else {
            (None, None)
        };

        Ok(Self {
            og_title: title.clone(),
            og_description: description.clone(),
            title,
            description,
            og_type: None,
            og_image: None,
        })
    }
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    avatar_url: Option<String>,
    instance_name: Option<String>,
    ctx: FrontendContext,
}

pub enum RespOrFrontend<T> {
    Resp(T),
    Frontend(StatusCode, IndexTemplate),
}

impl<T> RespOrFrontend<T> {
    pub fn resp(resp: T) -> Self {
        Self::Resp(resp)
    }

    pub async fn frontend(
        status: StatusCode,
        db: &impl ConnectionTrait,
        ctx: FrontendContext,
    ) -> Result<Self> {
        let setting = setting::Entity::find_by_id(Ulid::nil())
            .one(db)
            .await
            .context_internal_server_error("failed to query database")?;

        let (avatar_url, instance_name) = if let Some(setting) = setting {
            let avatar_url = if let Some(file_id) = setting.avatar_file_id {
                let url = local_file::Entity::find_by_id(file_id)
                    .select_only()
                    .column(local_file::Column::Url)
                    .into_tuple::<String>()
                    .one(db)
                    .await
                    .context_internal_server_error("failed to query database")?
                    .context_internal_server_error("file not found")?;
                Some(url)
            } else {
                None
            };

            (avatar_url, Some(setting.instance_name))
        } else {
            (None, None)
        };

        Ok(Self::Frontend(
            status,
            IndexTemplate {
                avatar_url,
                instance_name,
                ctx,
            },
        ))
    }
}

impl<T> IntoResponse for RespOrFrontend<T>
where
    T: IntoResponse,
{
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Resp(resp) => resp.into_response(),
            Self::Frontend(status, template) => {
                let mut resp = askama_axum::into_response(&template);
                *resp.status_mut() = status;
                resp
            }
        }
    }
}

pub async fn get_index(data: Data<State>) -> Result<RespOrFrontend<()>> {
    RespOrFrontend::frontend(
        StatusCode::OK,
        &*data.db,
        FrontendContext::site_default(&*data.db).await?,
    )
    .await
}

pub async fn get_not_found(data: Data<State>) -> Result<RespOrFrontend<()>> {
    RespOrFrontend::frontend(
        StatusCode::NOT_FOUND,
        &*data.db,
        FrontendContext::site_default(&*data.db).await?,
    )
    .await
}
