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
                    .table(Setting::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Setting::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Setting::InstanceName).string())
                    .col(ColumnDef::new(Setting::UserName).string())
                    .col(
                        ColumnDef::new(Setting::UserPublicKey)
                            .string_len(4096)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Setting::UserPrivateKey)
                            .string_len(4096)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Setting::AvatarFileId).uuid())
                    .col(ColumnDef::new(Setting::BannerFileId).uuid())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Setting::Table, Setting::AvatarFileId)
                            .to(LocalFile::Table, LocalFile::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Setting::Table, Setting::BannerFileId)
                            .to(LocalFile::Table, LocalFile::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Setting::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
pub enum Setting {
    Table,
    Id,
    InstanceName,
    UserName,
    UserPublicKey,
    UserPrivateKey,
    AvatarFileId,
    BannerFileId,
    InstanceDescription,
    MaintainerName,
    MaintainerEmail,
    ThemeColor,
    UserDescription,
}
