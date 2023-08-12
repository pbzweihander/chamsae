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
                    .table(Mention::Table)
                    .col(ColumnDef::new(Mention::PostId).uuid().not_null())
                    .col(ColumnDef::new(Mention::UserUri).string().not_null())
                    .col(ColumnDef::new(Mention::Name).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Mention::Table, Mention::PostId)
                            .to(Post::Table, Post::Id),
                    )
                    .index(
                        Index::create()
                            .col(Mention::PostId)
                            .col(Mention::UserUri)
                            .primary(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Mention::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Mention {
    Table,
    PostId,
    UserUri,
    Name,
}
