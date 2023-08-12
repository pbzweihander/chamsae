use activitypub_federation::{config::Data, traits::Object};
use axum::{extract, routing, Json, Router};
use chrono::Utc;
use futures_util::{stream::FuturesOrdered, TryStreamExt};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, ModelTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, TransactionTrait,
};
use ulid::Ulid;
use url::Url;

use crate::{
    ap::{delete::Delete, like::Like, undo::Undo},
    dto::{CreatePost, CreateReaction, IdPaginationQuery, IdResponse, Post, Visibility},
    entity::{
        emoji, hashtag, local_file, mention, post, post_emoji, reaction, sea_orm_active_enums, user,
    },
    error::{Context, Result},
    format_err,
    state::State,
    util::get_follower_inboxes,
};

use super::auth::Access;

pub(super) fn create_router() -> Router {
    Router::new()
        .route("/", routing::get(get_posts).post(post_post))
        .route("/:id", routing::get(get_post).delete(delete_post))
        .route(
            "/:id/reaction",
            routing::post(post_reaction).delete(delete_reaction),
        )
}

#[utoipa::path(
    get,
    path = "/api/post",
    params(IdPaginationQuery),
    responses(
        (status = 200, body = Vec<Post>),
    ),
    security(
        ("access_key" = []),
    ),
)]
#[tracing::instrument(skip(data, _access))]
async fn get_posts(
    data: Data<State>,
    _access: Access,
    extract::Query(query): extract::Query<IdPaginationQuery>,
) -> Result<Json<Vec<Post>>> {
    let pagination_query = post::Entity::find();
    let pagination_query = if let Some(after) = query.after {
        pagination_query.filter(post::Column::Id.lt(uuid::Uuid::from(after)))
    } else {
        pagination_query
    };
    let posts = pagination_query
        .order_by_desc(post::Column::Id)
        .limit(100)
        .all(&*data.db)
        .await
        .context_internal_server_error("failed to query database")?;
    let posts = posts
        .into_iter()
        .map(|post| Post::from_model(post, &*data.db))
        .collect::<FuturesOrdered<_>>()
        .try_collect()
        .await?;
    Ok(Json(posts))
}

#[utoipa::path(
    post,
    path = "/api/post",
    request_body = CreatePost,
    responses(
        (status = 200, body = IdResponse),
    ),
    security(
        ("access_key" = []),
    ),
)]
#[tracing::instrument(skip(data, _access, req))]
async fn post_post(
    data: Data<State>,
    _access: Access,
    Json(req): Json<CreatePost>,
) -> Result<Json<IdResponse>> {
    let tx = data
        .db
        .begin()
        .await
        .context_internal_server_error("failed to begin database transaction")?;

    if let Some(reply_id) = req.reply_id {
        let reply_post_count = post::Entity::find_by_id(reply_id)
            .count(&tx)
            .await
            .context_internal_server_error("failed to request database")?;
        if reply_post_count == 0 {
            return Err(format_err!(NOT_FOUND, "reply target post not found"));
        }
    }

    let emojis = emoji::Entity::find()
        .filter(emoji::Column::Name.is_in(req.emojis))
        .find_also_related(local_file::Entity)
        .all(&tx)
        .await
        .context_internal_server_error("failed to query database")?;

    let id = Ulid::new();
    let post_activemodel = post::ActiveModel {
        id: ActiveValue::Set(id.into()),
        created_at: ActiveValue::Set(Utc::now().fixed_offset()),
        reply_id: ActiveValue::Set(req.reply_id.map(Into::into)),
        text: ActiveValue::Set(req.text),
        title: ActiveValue::Set(req.title),
        user_id: ActiveValue::Set(None),
        visibility: ActiveValue::Set(match req.visibility {
            Visibility::Public => sea_orm_active_enums::Visibility::Public,
            Visibility::Home => sea_orm_active_enums::Visibility::Home,
            Visibility::Followers => sea_orm_active_enums::Visibility::Followers,
            Visibility::DirectMessage => sea_orm_active_enums::Visibility::DirectMessage,
        }),
        is_sensitive: ActiveValue::Set(req.is_sensitive),
        uri: ActiveValue::Set(post::Model::ap_id_from_id(id)?.to_string()),
    };
    let post = post_activemodel
        .insert(&tx)
        .await
        .context_internal_server_error("failed to insert to database")?;

    for (idx, local_file_id) in req.files.into_iter().enumerate() {
        let file = local_file::Entity::find_by_id(local_file_id)
            .one(&tx)
            .await
            .context_internal_server_error("failed to query database")?
            .context_not_found("file not found")?;
        file.attach_to_post(post.id.into(), idx as u8, &tx).await?;
    }

    let emojis = emojis
        .into_iter()
        .filter_map(|(emoji, file)| file.map(|file| (emoji, file)))
        .filter_map(|(emoji, file)| {
            Some(post_emoji::ActiveModel {
                post_id: ActiveValue::Set(post.id),
                uri: ActiveValue::Set(emoji.ap_id().ok()?.to_string()),
                name: ActiveValue::Set(emoji.name),
                media_type: ActiveValue::Set(file.media_type),
                image_url: ActiveValue::Set(file.url),
            })
        })
        .collect::<Vec<_>>();
    if !emojis.is_empty() {
        post_emoji::Entity::insert_many(emojis)
            .exec(&tx)
            .await
            .context_internal_server_error("failed to insert to database")?;
    }

    let mentions = req
        .mentions
        .iter()
        .map(|mention| mention::ActiveModel {
            post_id: ActiveValue::Set(post.id),
            user_uri: ActiveValue::Set(mention.user_uri.to_string()),
            name: ActiveValue::Set(mention.name.clone()),
        })
        .collect::<Vec<_>>();
    if !mentions.is_empty() {
        mention::Entity::insert_many(mentions)
            .exec(&tx)
            .await
            .context_internal_server_error("failed to insert to database")?;
    }

    let hashtags = req
        .hashtags
        .into_iter()
        .map(|hashtag| hashtag::ActiveModel {
            post_id: ActiveValue::Set(post.id),
            name: ActiveValue::Set(hashtag),
        })
        .collect::<Vec<_>>();
    if !hashtags.is_empty() {
        hashtag::Entity::insert_many(hashtags)
            .exec(&tx)
            .await
            .context_internal_server_error("failed to insert to database")?;
    }

    tx.commit()
        .await
        .context_internal_server_error("failed to commmit database transaction")?;

    let post_id = post.id.into();
    let visibility = post.visibility.clone();

    let post = post.into_json(&data).await?;

    let inboxes = match visibility {
        sea_orm_active_enums::Visibility::Public
        | sea_orm_active_enums::Visibility::Home
        | sea_orm_active_enums::Visibility::Followers => get_follower_inboxes(&*data.db).await?,
        sea_orm_active_enums::Visibility::DirectMessage => req
            .mentions
            .into_iter()
            .map(|mention| mention.user_uri)
            .collect::<Vec<_>>(),
    };

    let create = post.into_create()?;
    create.send(&data, inboxes).await?;

    Ok(Json(IdResponse { id: post_id }))
}

#[utoipa::path(
    get,
    path = "/api/post/{id}",
    params(
        ("id" = String, format = "ulid"),
    ),
    responses(
        (status = 200, body = Post),
    ),
    security(
        ("access_key" = []),
    ),
)]
#[tracing::instrument(skip(data, _access))]
async fn get_post(
    data: Data<State>,
    extract::Path(id): extract::Path<Ulid>,
    _access: Access,
) -> Result<Json<Post>> {
    let post = post::Entity::find_by_id(id)
        .one(&*data.db)
        .await
        .context_internal_server_error("failed to query database")?
        .context_not_found("post not found")?;
    Ok(Json(Post::from_model(post, &*data.db).await?))
}

#[utoipa::path(
    delete,
    path = "/api/post/{id}",
    params(
        ("id" = String, format = "ulid"),
    ),
    responses(
        (status = 200),
    ),
    security(
        ("access_key" = []),
    ),
)]
#[tracing::instrument(skip(data, _access))]
async fn delete_post(
    data: Data<State>,
    extract::Path(id): extract::Path<Ulid>,
    _access: Access,
) -> Result<()> {
    let tx = data
        .db
        .begin()
        .await
        .context_internal_server_error("failed to begin database transaction")?;

    let existing = post::Entity::find_by_id(id)
        .one(&tx)
        .await
        .context_internal_server_error("failed to query database")?;

    if let Some(existing) = existing {
        let was_mine = existing.user_id.is_none();
        let visibility = existing.visibility.clone();
        let mention_user_uris = existing
            .find_related(mention::Entity)
            .select_only()
            .column(mention::Column::UserUri)
            .into_tuple::<String>()
            .all(&tx)
            .await
            .context_internal_server_error("failed to query database")?;
        let mention_user_uris = mention_user_uris
            .into_iter()
            .filter_map(|uri| Url::parse(&uri).ok())
            .collect::<Vec<_>>();
        let uri = existing.uri.clone();

        ModelTrait::delete(existing, &tx)
            .await
            .context_internal_server_error("failed to delete from database")?;

        tx.commit()
            .await
            .context_internal_server_error("failed to commit database transaction")?;

        if was_mine {
            let inboxes = match visibility {
                sea_orm_active_enums::Visibility::Public
                | sea_orm_active_enums::Visibility::Home
                | sea_orm_active_enums::Visibility::Followers => {
                    get_follower_inboxes(&*data.db).await?
                }
                sea_orm_active_enums::Visibility::DirectMessage => mention_user_uris,
            };

            let delete = Delete::new(
                uri.parse()
                    .context_internal_server_error("malformed post URI")?,
            )?;
            delete.send(&data, inboxes).await?;
        }

        Ok(())
    } else {
        Ok(())
    }
}

#[utoipa::path(
    post,
    path = "/api/post/{id}/reaction",
    params(
        ("id" = String, format = "ulid"),
    ),
    request_body = CreateReaction,
    responses(
        (status = 200),
    ),
    security(
        ("access_key" = []),
    ),
)]
#[tracing::instrument(skip(data, _access))]
async fn post_reaction(
    data: Data<State>,
    _access: Access,
    extract::Path(id): extract::Path<Ulid>,
    Json(req): Json<CreateReaction>,
) -> Result<()> {
    let tx = data
        .db
        .begin()
        .await
        .context_internal_server_error("failed to begin database transaction")?;

    let existing_post_count = post::Entity::find_by_id(id)
        .count(&tx)
        .await
        .context_internal_server_error("failed to query database")?;

    if existing_post_count == 0 {
        return Err(format_err!(NOT_FOUND, "post not found"));
    }

    let existing_reaction_count = reaction::Entity::find()
        .filter(
            reaction::Column::PostId
                .eq(uuid::Uuid::from(id))
                .and(reaction::Column::UserId.is_null()),
        )
        .count(&tx)
        .await
        .context_internal_server_error("failed to query database")?;

    if existing_reaction_count > 0 {
        return Err(format_err!(CONFLICT, "already reacted post"));
    }

    let (content, emoji_uri, emoji_media_type, emoji_image_url) = match req {
        CreateReaction::Emoji(req) => {
            let (emoji, file) = emoji::Entity::find_by_id(req.emoji_name)
                .find_also_related(local_file::Entity)
                .one(&tx)
                .await
                .context_internal_server_error("failed to query database")?
                .context_not_found("emoji not found")?;
            let file = file.context_internal_server_error("failed to find emoji file")?;
            (
                format!(":{}:", emoji.name),
                Some(emoji.ap_id()?.to_string()),
                Some(file.media_type),
                Some(file.url),
            )
        }
        CreateReaction::Content(req) => (req.content, None, None, None),
    };

    let reaction_id = Ulid::new();
    let reaction_activemodel = reaction::ActiveModel {
        id: ActiveValue::Set(reaction_id.into()),
        user_id: ActiveValue::Set(None),
        post_id: ActiveValue::Set(id.into()),
        content: ActiveValue::Set(content),
        uri: ActiveValue::Set(reaction::Model::ap_id_from_id(reaction_id)?.to_string()),
        emoji_uri: ActiveValue::Set(emoji_uri),
        emoji_media_type: ActiveValue::Set(emoji_media_type),
        emoji_image_url: ActiveValue::Set(emoji_image_url),
    };
    let reaction = reaction_activemodel
        .insert(&tx)
        .await
        .context_internal_server_error("failed to insert to database")?;

    tx.commit()
        .await
        .context_internal_server_error("failed to commit database transation")?;

    let like = reaction.into_json(&data).await?;
    like.send(&data).await?;

    Ok(())
}

#[utoipa::path(
    delete,
    path = "/api/post/{id}/reaction",
    params(
        ("id" = String, format = "ulid"),
    ),
    responses(
        (status = 200),
    ),
    security(
        ("access_key" = []),
    ),
)]
#[tracing::instrument(skip(data, _access))]
async fn delete_reaction(
    data: Data<State>,
    _access: Access,
    extract::Path(id): extract::Path<Ulid>,
) -> Result<()> {
    let tx = data
        .db
        .begin()
        .await
        .context_internal_server_error("failed to begin database transaction")?;

    let existing = reaction::Entity::find()
        .filter(
            reaction::Column::PostId
                .eq(uuid::Uuid::from(id))
                .and(reaction::Column::UserId.is_null()),
        )
        .one(&tx)
        .await
        .context_internal_server_error("failed to query database")?;

    if let Some(existing) = existing {
        let inbox = existing
            .find_related(post::Entity)
            .inner_join(user::Entity)
            .select_only()
            .column(user::Column::Inbox)
            .into_tuple::<String>()
            .one(&*data.db)
            .await
            .context_internal_server_error("failed to query database")?
            .context_internal_server_error("user not found")?;
        let like = existing.clone().into_json(&data).await?;

        ModelTrait::delete(existing, &tx)
            .await
            .context_internal_server_error("failed to delete from database")?;

        tx.commit()
            .await
            .context_internal_server_error("failed to commit database transaction")?;

        let inbox = Url::parse(&inbox).context_internal_server_error("malformed user inbox URL")?;
        let undo = Undo::<Like, reaction::Model>::new(like)?;
        undo.send(&data, vec![inbox]).await?;
    }

    Ok(())
}
