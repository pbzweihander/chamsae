use sea_orm_migration::prelude::*;

use crate::m20230806_104639_initial::Post;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(LocalFile::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(LocalFile::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(LocalFile::PostId).uuid())
                    .col(ColumnDef::new(LocalFile::Order).tiny_unsigned())
                    .col(
                        ColumnDef::new(LocalFile::ObjectStorageKey)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(LocalFile::MediaType).string().not_null())
                    .col(ColumnDef::new(LocalFile::Url).string().not_null())
                    .col(ColumnDef::new(LocalFile::Alt).string())
                    .foreign_key(
                        ForeignKey::create()
                            .from(LocalFile::Table, LocalFile::PostId)
                            .to(Post::Table, Post::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .index(
                        Index::create()
                            .table(LocalFile::Table)
                            .col(LocalFile::PostId)
                            .col(LocalFile::Order)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(LocalFile::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum LocalFile {
    Table,
    Id,
    PostId,
    Order,
    ObjectStorageKey,
    MediaType,
    Url,
    Alt,
}
