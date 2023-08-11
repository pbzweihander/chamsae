use sea_orm_migration::prelude::*;

use crate::m20230806_104639_initial::{Post, User};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Reaction::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Reaction::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Reaction::UserId).uuid())
                    .col(ColumnDef::new(Reaction::PostId).uuid().not_null())
                    .col(ColumnDef::new(Reaction::Content).string().not_null())
                    .col(
                        ColumnDef::new(Reaction::Uri)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Reaction::EmojiUri).string())
                    .col(ColumnDef::new(Reaction::EmojiMediaType).string())
                    .col(ColumnDef::new(Reaction::EmojiImageUrl).string())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Reaction::Table, Reaction::UserId)
                            .to(User::Table, User::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Reaction::Table, Reaction::PostId)
                            .to(Post::Table, Post::Id),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Reaction::Table).to_owned())
            .await?;

        Ok(())
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Reaction {
    Table,
    Id,
    UserId,
    PostId,
    Content,
    Uri,
    EmojiUri,
    EmojiMediaType,
    EmojiImageUrl,
}
