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
                    .col(
                        ColumnDef::new(Emoji::Name)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Emoji::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(LocalFile::Table)
                    .add_column(ColumnDef::new(LocalFile::EmojiName).string())
                    .add_foreign_key(
                        ForeignKey::create()
                            .from(LocalFile::Table, LocalFile::EmojiName)
                            .to(Emoji::Table, Emoji::Name)
                            .on_delete(ForeignKeyAction::SetNull)
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
                    .drop_column(LocalFile::EmojiName)
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
    Name,
    CreatedAt,
}
