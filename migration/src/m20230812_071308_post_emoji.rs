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
                    .table(PostEmoji::Table)
                    .col(ColumnDef::new(PostEmoji::PostId).uuid().not_null())
                    .col(ColumnDef::new(PostEmoji::Name).string().not_null())
                    .col(ColumnDef::new(PostEmoji::Uri).string().not_null())
                    .col(ColumnDef::new(PostEmoji::MediaType).string().not_null())
                    .col(ColumnDef::new(PostEmoji::ImageUrl).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(PostEmoji::Table, PostEmoji::PostId)
                            .to(Post::Table, Post::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .col(PostEmoji::PostId)
                            .col(PostEmoji::Name)
                            .primary(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PostEmoji::Table).to_owned())
            .await?;

        Ok(())
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum PostEmoji {
    Table,
    PostId,
    Name,
    Uri,
    MediaType,
    ImageUrl,
}
