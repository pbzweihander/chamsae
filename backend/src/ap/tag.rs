use activitypub_federation::kinds::{link::MentionType, object::ImageType};
use mime::Mime;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Mention {
    #[serde(rename = "type")]
    pub ty: MentionType,
    pub href: Url,
    pub name: String,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum HashtagType {
    Hashtag,
}

impl std::fmt::Display for HashtagType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Hashtag")
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Hashtag {
    #[serde(rename = "type")]
    pub ty: HashtagType,
    pub href: Url,
    pub name: String,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum EmojiType {
    Emoji,
}

impl std::fmt::Display for EmojiType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Emoji")
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmojiIcon {
    #[serde(rename = "type")]
    pub ty: ImageType,
    #[serde(with = "mime_serde_shim")]
    pub media_type: Mime,
    pub url: Url,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Emoji {
    #[serde(rename = "type")]
    pub ty: EmojiType,
    pub id: Url,
    pub name: String,
    pub icon: EmojiIcon,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Tag {
    Mention(Mention),
    Hashtag(Hashtag),
    Emoji(Emoji),
}
