use sea_orm_migration::prelude::*;

use crate::m20230811_163629_local_file::LocalFile;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Emoji::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Emoji::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Emoji::Name).string().not_null().unique_key())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(LocalFile::Table)
                    .add_column(ColumnDef::new(LocalFile::EmojiId).uuid())
                    .add_foreign_key(
                        ForeignKey::create()
                            .name("local_file_emoji_id_fk")
                            .from(LocalFile::Table, LocalFile::EmojiId)
                            .to(Emoji::Table, Emoji::Id)
                            .get_foreign_key(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(LocalFile::Table)
                    .drop_column(LocalFile::EmojiId)
                    .drop_foreign_key(Alias::new("local_file_emoji_id_fk"))
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Emoji::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Emoji {
    Table,
    Id,
    Name,
}
