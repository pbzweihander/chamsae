use sea_orm_migration::prelude::*;

use crate::{
    m20230812_135017_setting::Setting, m20230813_081932_object_store_type::ObjectStoreType,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Setting::Table)
                    .add_column(ColumnDef::new(Setting::ObjectStoreType).enumeration(
                        ObjectStoreType::Table,
                        [ObjectStoreType::S3, ObjectStoreType::LocalFileSystem],
                    ))
                    .add_column(
                        ColumnDef::new(Setting::ObjectStoreS3Bucket).string().check(
                            Expr::col(Setting::ObjectStoreType)
                                .eq("s3")
                                .and(Expr::col(Setting::ObjectStoreS3Bucket).is_not_null())
                                .or(Expr::col(Setting::ObjectStoreType).ne("s3")),
                        ),
                    )
                    .add_column(
                        ColumnDef::new(Setting::ObjectStoreS3PublicUrlBase)
                            .string()
                            .check(
                                Expr::col(Setting::ObjectStoreType)
                                    .eq("s3")
                                    .and(
                                        Expr::col(Setting::ObjectStoreS3PublicUrlBase)
                                            .is_not_null(),
                                    )
                                    .or(Expr::col(Setting::ObjectStoreType).ne("s3")),
                            ),
                    )
                    .add_column(
                        ColumnDef::new(Setting::ObjectStoreLocalFileSystemBasePath)
                            .string()
                            .check(
                                Expr::col(Setting::ObjectStoreType)
                                    .eq("local_file_system")
                                    .and(
                                        Expr::col(Setting::ObjectStoreLocalFileSystemBasePath)
                                            .is_not_null(),
                                    )
                                    .or(Expr::col(Setting::ObjectStoreType).ne("local_file_system")),
                            ),
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
                    .table(Setting::Table)
                    .drop_column(Setting::ObjectStoreType)
                    .drop_column(Setting::ObjectStoreS3Bucket)
                    .drop_column(Setting::ObjectStoreS3PublicUrlBase)
                    .drop_column(Setting::ObjectStoreLocalFileSystemBasePath)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
