use activitypub_federation::kinds::{link::MentionType, object::ImageType};
use derivative::Derivative;
use mime::Mime;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Derivative, Deserialize, Serialize)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct Mention {
    #[serde(rename = "type")]
    pub ty: MentionType,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
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

impl Default for HashtagType {
    fn default() -> Self {
        Self::Hashtag
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Hashtag {
    #[serde(rename = "type")]
    pub ty: HashtagType,
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

impl Default for EmojiType {
    fn default() -> Self {
        Self::Emoji
    }
}

#[derive(Clone, Derivative, Deserialize, Serialize)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct EmojiIcon {
    #[serde(rename = "type")]
    pub ty: ImageType,
    #[serde(with = "mime_serde_shim")]
    pub media_type: Mime,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub url: Url,
}

#[derive(Clone, Derivative, Deserialize, Serialize)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct Emoji {
    #[serde(rename = "type")]
    pub ty: EmojiType,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub id: Url,
    pub name: String,
    pub icon: EmojiIcon,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Tag {
    Mention(Mention),
    Hashtag(Hashtag),
    Emoji(Emoji),
}
