use std::convert::Infallible;

use axum::response::sse::Event;
use futures_util::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use stopper::Stopper;
use ulid::Ulid;
use utoipa::ToSchema;

use crate::error::Error;

const NOTIFICATION_CHANNEL_NAME: &str = "notification";

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Notification {
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

impl Notification {
    pub async fn send(self, redis: &mut impl redis::AsyncCommands) -> crate::error::Result<()> {
        use crate::error::Context;

        let payload = serde_json::to_vec(&self)
            .context_internal_server_error("failed to serialize Redis channel payload")?;
        redis
            .publish(NOTIFICATION_CHANNEL_NAME, payload)
            .await
            .context_internal_server_error("failed to publish to Redis channel")?;
        Ok(())
    }
}

fn make_event(msg: redis::Msg) -> anyhow::Result<Event> {
    use anyhow::Context;

    let payload = msg.get_payload_bytes();
    let payload: Notification =
        serde_json::from_slice(payload).context("failed to deserialize Redis channel payload")?;
    let event = Event::default()
        .json_data(payload)
        .context("failed to construct SSE event")?;
    Ok(event)
}

pub async fn notification_stream(
    mut pubsub: redis::aio::PubSub,
    stopper: Stopper,
) -> Result<impl Stream<Item = Result<Event, Infallible>>, Error> {
    use crate::error::Context;

    pubsub
        .subscribe(NOTIFICATION_CHANNEL_NAME)
        .await
        .context_internal_server_error("failed to subscribe Redis channel")?;
    let stream = stopper
        .stop_stream(pubsub.into_on_message())
        .filter_map(|msg| {
            let opt = match make_event(msg) {
                Ok(event) => Some(Result::<_, Infallible>::Ok(event)),
                Err(error) => {
                    tracing::error!("failed to make SSE event\n{:?}", error);
                    None
                }
            };
            async move { opt }
        });
    Ok(stream)
}
