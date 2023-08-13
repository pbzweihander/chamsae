use sea_orm_migration::{prelude::*, sea_query::extension::postgres::Type};

use crate::m20230811_163629_local_file::LocalFile;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(ObjectStoreType::Table)
                    .values([ObjectStoreType::S3, ObjectStoreType::LocalFileSystem])
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(LocalFile::Table)
                    .rename_column(LocalFile::ObjectStorageKey, LocalFile::ObjectStoreKey)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(LocalFile::Table)
                    .add_column(
                        ColumnDef::new(LocalFile::ObjectStoreType)
                            .enumeration(
                                ObjectStoreType::Table,
                                [ObjectStoreType::S3, ObjectStoreType::LocalFileSystem],
                            )
                            .not_null()
                            .default("s3"),
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
                    .drop_column(LocalFile::ObjectStoreType)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(LocalFile::Table)
                    .rename_column(LocalFile::ObjectStoreKey, LocalFile::ObjectStorageKey)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_type(Type::drop().name(ObjectStoreType::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum ObjectStoreType {
    Table,
    S3,
    LocalFileSystem,
}
