use activitypub_federation::{
    config::Data,
    fetch::webfinger::{build_webfinger_response, extract_webfinger_name, Webfinger},
};
use axum::{extract, routing, Json, Router};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    ap::person::LocalPerson,
    config::CONFIG,
    entity::setting,
    error::{Context, Result},
    format_err,
    state::State,
};

pub(super) fn create_router() -> Router {
    Router::new()
        .route("/webfinger", routing::get(get_webfinger))
        .route("/nodeinfo", routing::get(get_nodeinfo))
}

#[derive(Debug, Deserialize)]
struct GetWebfingerQuery {
    resource: String,
}

#[tracing::instrument(skip(data))]
async fn get_webfinger(
    extract::Query(query): extract::Query<GetWebfingerQuery>,
    data: Data<State>,
) -> Result<Json<Webfinger>> {
    let setting = setting::Model::get(&*data.db).await?;
    let name = extract_webfinger_name(&query.resource, &data)
        .context_bad_request("failed to extract resource name")?;
    if name == setting.user_handle {
        let resp = build_webfinger_response(name.to_string(), LocalPerson::id());
        Ok(Json(resp))
    } else {
        Err(format_err!(NOT_FOUND, "user not found"))
    }
}

#[derive(Debug, Serialize)]
struct NodeInfoWellKnownLinks {
    rel: Url,
    href: Url,
}

#[derive(Debug, Serialize)]
struct NodeInfoWellKnown {
    links: Vec<NodeInfoWellKnownLinks>,
}

#[tracing::instrument]
async fn get_nodeinfo() -> Result<Json<NodeInfoWellKnown>> {
    Ok(Json(NodeInfoWellKnown {
        links: vec![NodeInfoWellKnownLinks {
            rel: Url::parse("http://nodeinfo.diaspora.software/ns/schema/2.0")
                .context_internal_server_error("failed to construct URL")?,
            href: Url::parse(&format!("https://{}/nodeinfo/2.0", CONFIG.public_domain,))
                .context_internal_server_error("failed to construct URL")?,
        }],
    }))
}
