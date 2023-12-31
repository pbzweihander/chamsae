use sea_orm::{
    sea_query::{Expr, Func},
    ConnectionTrait, EntityTrait, QuerySelect,
};
use url::Url;

use crate::{
    entity::{follower, user},
    error::{Context, Result},
};

pub async fn get_follower_inboxes(db: &impl ConnectionTrait) -> Result<Vec<Url>> {
    let inboxes = follower::Entity::find()
        .inner_join(user::Entity)
        .select_only()
        .expr(Func::coalesce([
            Expr::col(user::Column::SharedInbox).into(),
            Expr::col(user::Column::Inbox).into(),
        ]))
        .distinct()
        .into_tuple::<String>()
        .all(db)
        .await
        .context_internal_server_error("failed to query database")?;
    let inboxes = inboxes
        .into_iter()
        .filter_map(|url| Url::parse(&url).ok())
        .collect::<Vec<_>>();
    Ok(inboxes)
}
