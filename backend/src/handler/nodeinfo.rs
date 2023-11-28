use activitypub_federation::config::Data;
use axum::Json;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};
use serde::{Deserialize, Serialize};

use crate::{
    entity::{post, setting},
    error::Result,
    state::State,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NodeInfoSoftware {
    name: String,
    version: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeInfoUsageUsers {
    total: usize,
    active_month: usize,
    active_half_year: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeInfoUsage {
    users: NodeInfoUsageUsers,
    local_posts: u64,
    local_comments: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeInfoMetadataMaintainer {
    name: Option<String>,
    email: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeInfoMetadata {
    node_name: String,
    node_description: Option<String>,
    maintainer: NodeInfoMetadataMaintainer,
    theme_color: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeInfo {
    version: String,
    software: NodeInfoSoftware,
    protocols: Vec<String>,
    usage: NodeInfoUsage,
    open_registrations: bool,
    metadata: NodeInfoMetadata,
}

// TODO: cache
pub async fn get_nodeinfo_2_0(data: Data<State>) -> Result<Json<NodeInfo>> {
    let setting = setting::Model::get(&*data.db).await?;
    let local_post_count = post::Entity::find()
        .filter(post::Column::UserId.is_null())
        .count(&*data.db)
        .await?;

    let nodeinfo = NodeInfo {
        version: "2.0".to_string(),
        software: NodeInfoSoftware {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        protocols: vec!["activitypub".to_string()],
        usage: NodeInfoUsage {
            users: NodeInfoUsageUsers {
                total: 1,
                active_month: 1,
                active_half_year: 1,
            },
            local_posts: local_post_count,
            local_comments: 0,
        },
        open_registrations: false,
        metadata: NodeInfoMetadata {
            node_name: setting.instance_name,
            node_description: setting.instance_description,
            maintainer: NodeInfoMetadataMaintainer {
                name: setting.maintainer_name,
                email: setting.maintainer_email,
            },
            theme_color: setting.theme_color,
        },
    };

    Ok(Json(nodeinfo))
}
