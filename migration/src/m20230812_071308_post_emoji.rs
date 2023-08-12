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
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PostEmoji::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PostEmoji::PostId).uuid().not_null())
                    .col(ColumnDef::new(PostEmoji::Name).string().not_null())
                    .col(ColumnDef::new(PostEmoji::Uri).string().not_null())
                    .col(ColumnDef::new(PostEmoji::MediaType).string().not_null())
                    .col(ColumnDef::new(PostEmoji::ImageUrl).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(PostEmoji::Table, PostEmoji::PostId)
                            .to(Post::Table, Post::Id),
                    )
                    .index(
                        Index::create()
                            .col(PostEmoji::PostId)
                            .col(PostEmoji::Name)
                            .unique(),
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
    Id,
    PostId,
    Name,
    Uri,
    MediaType,
    ImageUrl,
}
