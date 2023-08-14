use sea_orm_migration::prelude::*;

use crate::m20230806_104639_initial::Post;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Post::Table)
                    .add_column(ColumnDef::new(Post::RepostId).uuid())
                    .add_foreign_key(
                        ForeignKey::create()
                            .from(Post::Table, Post::RepostId)
                            .to(Post::Table, Post::Id)
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
                    .table(Post::Table)
                    .drop_column(Post::RepostId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
