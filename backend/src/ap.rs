use activitypub_federation::{
    fetch::object_id::ObjectId, kinds::actor::PersonType, protocol::public_key::PublicKey,
};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::entity::user;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    pub id: ObjectId<user::Model>,
    #[serde(rename = "type")]
    pub ty: PersonType,
    pub preferred_username: String,
    pub name: String,
    pub inbox: Url,
    pub public_key: PublicKey,
}
