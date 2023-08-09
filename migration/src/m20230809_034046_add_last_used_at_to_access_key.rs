use sea_orm_migration::prelude::*;

use crate::m20230806_104639_initial::AccessKey;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(AccessKey::Table)
                    .add_column(ColumnDef::new(AccessKey::LastUsedAt).timestamp_with_time_zone())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(AccessKey::Table)
                    .drop_column(AccessKey::LastUsedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
