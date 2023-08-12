use sea_orm_migration::prelude::*;

use crate::m20230812_135017_setting::Setting;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Setting::Table)
                    .add_column(ColumnDef::new(Setting::InstanceDescription).string())
                    .add_column(ColumnDef::new(Setting::MaintainerName).string())
                    .add_column(ColumnDef::new(Setting::MaintainerEmail).string())
                    .add_column(ColumnDef::new(Setting::ThemeColor).string())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Setting::Table)
                    .drop_column(Setting::InstanceDescription)
                    .drop_column(Setting::MaintainerName)
                    .drop_column(Setting::MaintainerEmail)
                    .drop_column(Setting::ThemeColor)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
