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
                    .table(Hashtag::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Hashtag::PostId).uuid().not_null())
                    .col(ColumnDef::new(Hashtag::Name).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Hashtag::Table, Hashtag::PostId)
                            .to(Post::Table, Post::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .col(Hashtag::PostId)
                            .col(Hashtag::Name)
                            .primary(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Hashtag::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Hashtag {
    Table,
    PostId,
    Name,
}
