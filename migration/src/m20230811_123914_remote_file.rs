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
                    .table(RemoteFile::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RemoteFile::Id)
                            .string_len(26)
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(RemoteFile::PostId).string_len(26).not_null())
                    .col(ColumnDef::new(RemoteFile::Order).tiny_unsigned().not_null())
                    .col(ColumnDef::new(RemoteFile::MediaType).string().not_null())
                    .col(ColumnDef::new(RemoteFile::Url).string().not_null())
                    .col(ColumnDef::new(RemoteFile::Alt).string())
                    .foreign_key(
                        ForeignKey::create()
                            .from(RemoteFile::Table, RemoteFile::PostId)
                            .to(Post::Table, Post::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .table(RemoteFile::Table)
                            .col(RemoteFile::PostId)
                            .col(RemoteFile::Order)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RemoteFile::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum RemoteFile {
    Table,
    Id,
    PostId,
    Order,
    MediaType,
    Url,
    Alt,
}
