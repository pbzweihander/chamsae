use sea_orm_migration::prelude::*;

use crate::m20230806_104639_initial::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .add_column(ColumnDef::new(User::AvatarUrl).string())
                    .add_column(ColumnDef::new(User::BannerUrl).string())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .drop_column(User::AvatarUrl)
                    .drop_column(User::BannerUrl)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
