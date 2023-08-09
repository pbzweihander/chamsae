use sea_orm_migration::{prelude::*, sea_query::extension::postgres::Type};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .col(
                        ColumnDef::new(User::Id)
                            .string_len(26)
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(User::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(User::LastFetchedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(User::Handle).string().not_null())
                    .col(ColumnDef::new(User::Name).string().not_null())
                    .col(ColumnDef::new(User::Host).string().not_null())
                    .col(ColumnDef::new(User::Inbox).string().not_null())
                    .col(ColumnDef::new(User::PublicKey).string_len(4096).not_null())
                    .col(ColumnDef::new(User::Uri).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(Visibility::Table)
                    .values([
                        Visibility::Public,
                        Visibility::Home,
                        Visibility::Followers,
                        Visibility::DirectMessage,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Post::Table)
                    .col(
                        ColumnDef::new(Post::Id)
                            .string_len(26)
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Post::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Post::ReplyId).string_len(26))
                    .col(ColumnDef::new(Post::RepostId).string_len(26))
                    .col(ColumnDef::new(Post::Text).string().not_null())
                    .col(ColumnDef::new(Post::Title).string())
                    .col(ColumnDef::new(Post::UserId).string_len(26))
                    .col(
                        ColumnDef::new(Post::RepostCount)
                            .unsigned()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Post::ReplyCount)
                            .unsigned()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(Post::Reactions).json_binary().not_null())
                    .col(
                        ColumnDef::new(Post::Visibility)
                            .enumeration(
                                Visibility::Table,
                                [
                                    Visibility::Public,
                                    Visibility::Home,
                                    Visibility::Followers,
                                    Visibility::DirectMessage,
                                ],
                            )
                            .not_null(),
                    )
                    .col(ColumnDef::new(Post::Uri).string())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Post::Table, Post::ReplyId)
                            .to(Post::Table, Post::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Post::Table, Post::RepostId)
                            .to(Post::Table, Post::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Post::Table, Post::UserId)
                            .to(User::Table, User::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AccessKey::Table)
                    .col(
                        ColumnDef::new(AccessKey::Id)
                            .string_len(26)
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AccessKey::Name).string().not_null())
                    .col(
                        ColumnDef::new(AccessKey::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(AccessKey::LastUsedAt).timestamp_with_time_zone())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AccessKey::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Post::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(Visibility::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum User {
    Table,
    Id,
    CreatedAt,
    LastFetchedAt,
    Handle,
    Name,
    Host,
    Inbox,
    PublicKey,
    Uri,
}

#[derive(Iden)]
enum Post {
    Table,
    Id,
    CreatedAt,
    ReplyId,
    RepostId,
    Text,
    Title,
    UserId,
    RepostCount,
    ReplyCount,
    Reactions,
    Visibility,
    Uri,
}

#[derive(Iden)]
enum Visibility {
    Table,
    Public,
    Home,
    Followers,
    DirectMessage,
}

#[derive(Iden)]
enum AccessKey {
    Table,
    Id,
    Name,
    CreatedAt,
    LastUsedAt,
}
