use activitypub_federation::config::Data;
use axum::{routing, Json, Router};
use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait, PaginatorTrait, TransactionTrait};

use crate::{
    ap::person::PersonUpdate,
    dto::Setting,
    entity::{local_file, setting},
    error::{Context, Result},
    format_err,
    state::State,
};

use super::auth::Access;

pub(super) fn create_router() -> Router {
    Router::new().route("/", routing::get(get_setting).put(put_setting))
}

async fn get_setting(data: Data<State>, _access: Access) -> Result<Json<Setting>> {
    let setting = setting::Model::get(&*data.db).await?;
    Ok(Json(Setting::from_model(setting)))
}

async fn put_setting(
    data: Data<State>,
    _access: Access,
    Json(req): Json<Setting>,
) -> Result<Json<Setting>> {
    let setting = setting::Model::get(&*data.db).await?;

    let mut setting_activemodel: setting::ActiveModel = setting.into();
    if let Some(v) = req.user_name {
        setting_activemodel.user_name = ActiveValue::Set(Some(v));
    }
    if let Some(v) = req.instance_name {
        setting_activemodel.instance_name = ActiveValue::Set(Some(v));
    }
    if let Some(v) = req.avatar_file_id {
        setting_activemodel.avatar_file_id = ActiveValue::Set(Some(v.into()));
    }
    if let Some(v) = req.banner_file_id {
        setting_activemodel.banner_file_id = ActiveValue::Set(Some(v.into()));
    }

    let tx = data
        .db
        .begin()
        .await
        .context_internal_server_error("failed to begin database transaction")?;

    if let Some(file_id) = req.avatar_file_id {
        let existing_file_count = local_file::Entity::find_by_id(file_id)
            .count(&tx)
            .await
            .context_internal_server_error("failed to query database")?;
        if existing_file_count == 0 {
            return Err(format_err!(NOT_FOUND, "file not found"));
        }
    }
    if let Some(file_id) = req.banner_file_id {
        let existing_file_count = local_file::Entity::find_by_id(file_id)
            .count(&tx)
            .await
            .context_internal_server_error("failed to query database")?;
        if existing_file_count == 0 {
            return Err(format_err!(NOT_FOUND, "file not found"));
        }
    }

    let setting = setting_activemodel
        .update(&tx)
        .await
        .context_internal_server_error("failed to update database")?;

    tx.commit()
        .await
        .context_internal_server_error("failed to commit database transaction")?;

    let update = PersonUpdate::new_self(&data).await?;
    update.send(&data).await?;

    Ok(Json(Setting::from_model(setting)))
}