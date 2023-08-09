//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.2

use super::sea_orm_active_enums::Visibility;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "post")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub created_at: TimeDateTimeWithTimeZone,
    pub reply_id: Option<String>,
    pub repost_id: Option<String>,
    pub text: String,
    pub title: Option<String>,
    pub user_id: Option<String>,
    pub repost_count: i32,
    pub reply_count: i32,
    #[sea_orm(column_type = "JsonBinary")]
    pub reactions: Json,
    pub visibility: Visibility,
    pub uri: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::file::Entity")]
    File,
    #[sea_orm(
        belongs_to = "Entity",
        from = "Column::ReplyId",
        to = "Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    SelfRef2,
    #[sea_orm(
        belongs_to = "Entity",
        from = "Column::RepostId",
        to = "Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    SelfRef1,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    User,
}

impl Related<super::file::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::File.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}