//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use super::sea_orm_active_enums::ObjectStoreType;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "local_file")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub post_id: Option<Uuid>,
    pub order: Option<i16>,
    pub object_store_key: String,
    pub media_type: String,
    pub url: String,
    pub alt: Option<String>,
    pub emoji_name: Option<String>,
    pub object_store_type: ObjectStoreType,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::emoji::Entity",
        from = "Column::EmojiName",
        to = "super::emoji::Column::Name",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    Emoji,
    #[sea_orm(
        belongs_to = "super::post::Entity",
        from = "Column::PostId",
        to = "super::post::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    Post,
}

impl Related<super::emoji::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Emoji.def()
    }
}

impl Related<super::post::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Post.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
