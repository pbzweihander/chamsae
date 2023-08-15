use sea_orm_migration::prelude::*;

use crate::m20230806_104639_initial::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Report::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Report::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Report::FromUserId).uuid().not_null())
                    .col(ColumnDef::new(Report::Content).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Report::Table, Report::FromUserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Report::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Report {
    Table,
    Id,
    FromUserId,
    Content,
}
