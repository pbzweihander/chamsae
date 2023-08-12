//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "setting")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub instance_name: Option<String>,
    pub user_name: Option<String>,
    pub user_public_key: String,
    pub user_private_key: String,
    pub avatar_file_id: Option<Uuid>,
    pub banner_file_id: Option<Uuid>,
    pub instance_description: Option<String>,
    pub maintainer_name: Option<String>,
    pub maintainer_email: Option<String>,
    pub theme_color: Option<String>,
    pub user_description: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::local_file::Entity",
        from = "Column::AvatarFileId",
        to = "super::local_file::Column::Id",
        on_update = "NoAction",
        on_delete = "Restrict"
    )]
    LocalFile2,
    #[sea_orm(
        belongs_to = "super::local_file::Entity",
        from = "Column::BannerFileId",
        to = "super::local_file::Column::Id",
        on_update = "NoAction",
        on_delete = "Restrict"
    )]
    LocalFile1,
}

impl ActiveModelBehavior for ActiveModel {}
