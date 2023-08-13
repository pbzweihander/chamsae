use std::convert::Infallible;

use anyhow::Context;
use async_stream::stream;
use axum::response::sse::Event;
use futures_util::Stream;
use pgmq::PGMQueue;
use serde::{Deserialize, Serialize};
use stopper::Stopper;
use ulid::Ulid;
use utoipa::ToSchema;

const DEFAULT_QUEUE_NAME: &str = "_default_queue";
const DEFAULT_VT: Option<i32> = Some(60); // 1 minute

pub async fn init_queue(queue: &PGMQueue) -> anyhow::Result<()> {
    queue
        .create(DEFAULT_QUEUE_NAME)
        .await
        .context("failed to create default message queue table")
}

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
    pub async fn send(self, queue: &PGMQueue) -> crate::error::Result<()> {
        crate::error::Context::context_internal_server_error(
            queue.send(DEFAULT_QUEUE_NAME, &self).await,
            "failed to send to message queue",
        )?;
        Ok(())
    }
}

async fn make_event(queue: &PGMQueue) -> anyhow::Result<Option<Event>> {
    if let Some(message) = queue
        .read::<Notification>(DEFAULT_QUEUE_NAME, DEFAULT_VT)
        .await
        .context("failed to read from message queue")?
    {
        let notification = message.message;
        let event = Event::default()
            .json_data(notification)
            .context("failed to construct SSE event")?;
        queue
            .delete(DEFAULT_QUEUE_NAME, message.msg_id)
            .await
            .context("failed to delete from message queue")?;
        Ok(Some(event))
    } else {
        Ok(None)
    }
}

pub fn notification_stream(
    queue: PGMQueue,
    stopper: Stopper,
) -> impl Stream<Item = Result<Event, Infallible>> {
    stream! {
        loop {
            match stopper.stop_future(make_event(&queue)).await {
                Some(Ok(Some(event))) => {
                    tracing::info!("foobar"); // TODO: remove
                    yield Ok(event);
                }
                Some(Ok(None)) => {
                    stopper.stop_future(tokio::time::sleep(std::time::Duration::from_secs(5))).await;
                }
                Some(Err(error)) => {
                    tracing::error!("failed to make SSE event\n{:?}", error);
                    stopper.stop_future(tokio::time::sleep(std::time::Duration::from_secs(10))).await;
                }
                None => {
                    break;
                }
            }
        }
    }
}
