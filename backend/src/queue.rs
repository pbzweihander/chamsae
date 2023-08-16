use std::convert::Infallible;

use axum::response::sse::Event as SseEvent;
use futures_util::{Stream, StreamExt};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ConnectionTrait, DbBackend, Statement, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use sqlx_postgres::{PgListener, PgNotification};
use ulid::Ulid;
use utoipa::ToSchema;

use crate::{entity::notification, error::Error};

const EVENT_CHANNEL_NAME: &str = "event";

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Update {
    #[serde(rename_all = "camelCase")]
    CreatePost {
        #[schema(value_type = String, format = "ulid")]
        post_id: Ulid,
    },
    #[serde(rename_all = "camelCase")]
    DeletePost {
        #[schema(value_type = String, format = "ulid")]
        post_id: Ulid,
    },
    #[serde(rename_all = "camelCase")]
    CreateReaction {
        #[schema(value_type = String, format = "ulid")]
        post_id: Ulid,
    },
    #[serde(rename_all = "camelCase")]
    DeleteReaction {
        #[schema(value_type = String, format = "ulid")]
        post_id: Ulid,
    },
    #[serde(rename_all = "camelCase")]
    UpdateUser {
        #[schema(value_type = String, format = "ulid")]
        user_id: Ulid,
    },
    #[serde(rename_all = "camelCase")]
    DeleteUser {
        #[schema(value_type = String, format = "ulid")]
        user_id: Ulid,
    },
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum NotificationType {
    #[serde(rename_all = "camelCase")]
    AcceptFollow {
        #[schema(value_type = String, format = "ulid")]
        user_id: Ulid,
    },
    #[serde(rename_all = "camelCase")]
    RejectFollow {
        #[schema(value_type = String, format = "ulid")]
        user_id: Ulid,
    },
    #[serde(rename_all = "camelCase")]
    CreateFollower {
        #[schema(value_type = String, format = "ulid")]
        user_id: Ulid,
    },
    #[serde(rename_all = "camelCase")]
    DeleteFollower {
        #[schema(value_type = String, format = "ulid")]
        user_id: Ulid,
    },
    #[serde(rename_all = "camelCase")]
    CreateReport {
        #[schema(value_type = String, format = "ulid")]
        report_id: Ulid,
    },
    #[serde(rename_all = "camelCase")]
    Mentioned {
        #[schema(value_type = String, format = "ulid")]
        post_id: Ulid,
    },
    #[serde(rename_all = "camelCase")]
    Reposted {
        #[schema(value_type = String, format = "ulid")]
        user_id: Ulid,
        #[schema(value_type = String, format = "ulid")]
        post_id: Ulid,
    },
    #[serde(rename_all = "camelCase")]
    Quoted {
        #[schema(value_type = String, format = "ulid")]
        post_id: Ulid,
    },
    #[serde(rename_all = "camelCase")]
    Reacted {
        #[schema(value_type = String, format = "ulid")]
        post_id: Ulid,
        #[schema(value_type = String, format = "ulid")]
        reaction_id: Ulid,
    },
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct Notification {
    #[schema(value_type = String, format = "ulid")]
    pub id: Ulid,
    #[serde(flatten)]
    pub ty: NotificationType,
}

impl Notification {
    pub fn new(ty: NotificationType) -> Self {
        Self {
            id: Ulid::new(),
            ty,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase", tag = "eventType")]
pub enum Event {
    Update(Update),
    Notification(Notification),
}

impl Event {
    #[tracing::instrument(skip(db))]
    pub async fn send(self, db: &impl TransactionTrait) -> crate::error::Result<()> {
        use crate::error::Context;

        let tx = db
            .begin()
            .await
            .context_internal_server_error("failed to begin database transaction")?;

        if let Event::Notification(notification) = &self {
            let payload = serde_json::to_value(&notification.ty)
                .context_internal_server_error("failed to serialize notification payload")?;

            let notification_activemodel = notification::ActiveModel {
                id: ActiveValue::Set(notification.id.into()),
                payload: ActiveValue::Set(payload),
            };
            notification_activemodel
                .insert(&tx)
                .await
                .context_internal_server_error("failed to insert to database")?;
        }

        let payload = serde_json::to_string(&self)
            .context_internal_server_error("failed to serialize Redis channel payload")?;

        // let statement = Statement::from_string(
        //     DbBackend::Postgres,
        //     format!("NOTIFY {}, '{}'", EVENT_CHANNEL_NAME, payload),
        // );
        let statement = Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT pg_notify($1, $2)",
            [EVENT_CHANNEL_NAME.into(), payload.into()],
        );
        tx.execute(statement)
            .await
            .context_internal_server_error("failed to notify to Postgres channel")?;

        tx.commit()
            .await
            .context_internal_server_error("failed to commit database transaction")?;

        Ok(())
    }
}

fn make_sse_event(msg: PgNotification) -> anyhow::Result<SseEvent> {
    use anyhow::Context;

    let payload = msg.payload();
    let payload: Event =
        serde_json::from_str(payload).context("failed to deserialize Redis channel payload")?;
    let event = SseEvent::default()
        .json_data(payload)
        .context("failed to construct SSE event")?;
    Ok(event)
}

pub async fn event_stream(
    mut pg_listener: PgListener,
) -> Result<impl Stream<Item = Result<SseEvent, Infallible>>, Error> {
    use crate::error::Context;

    pg_listener
        .listen(EVENT_CHANNEL_NAME)
        .await
        .context_internal_server_error("failed to listen Postgres channel")?;
    let stream = pg_listener.into_stream().filter_map(|msg| {
        let opt = match msg {
            Ok(msg) => match make_sse_event(msg) {
                Ok(event) => Some(Result::<_, Infallible>::Ok(event)),
                Err(error) => {
                    tracing::error!("failed to make SSE event\n{:?}", error);
                    None
                }
            },
            Err(error) => {
                tracing::error!("failed to listen from Postgres channel\n{:?}", error);
                None
            }
        };
        async move { opt }
    });
    Ok(stream)
}
