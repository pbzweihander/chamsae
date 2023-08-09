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
                    .col(ColumnDef::new(User::UpdatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(User::LastFetchedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(User::Handle).string().not_null())
                    .col(ColumnDef::new(User::Name).string().not_null())
                    .col(
                        ColumnDef::new(User::FollowerCount)
                            .unsigned()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(User::FollowingCount)
                            .unsigned()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(User::PostCount)
                            .unsigned()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(User::AvatarId).string_len(26).not_null())
                    .col(ColumnDef::new(User::BannerId).string_len(26).not_null())
                    .col(ColumnDef::new(User::IsBot).boolean().not_null())
                    .col(ColumnDef::new(User::Host).string().not_null())
                    .col(ColumnDef::new(User::Inbox).string().not_null())
                    .col(ColumnDef::new(User::PublicKey).string_len(4096).not_null())
                    .col(ColumnDef::new(User::PrivateKey).string_len(4096))
                    .col(ColumnDef::new(User::Uri).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(File::Table)
                    .col(
                        ColumnDef::new(File::Id)
                            .string_len(26)
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(File::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(File::UserId).string_len(26))
                    .col(ColumnDef::new(File::PostId).string_len(26))
                    .col(ColumnDef::new(File::Hash).string_len(64).not_null())
                    .col(ColumnDef::new(File::Mime).string().not_null())
                    .col(ColumnDef::new(File::Path).string().not_null())
                    .col(ColumnDef::new(File::Url).string().not_null())
                    .col(ColumnDef::new(File::ThumbnailUrl).string().not_null())
                    .col(ColumnDef::new(File::IsSensitive).boolean().not_null())
                    .col(ColumnDef::new(File::OriginalUrl).string())
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

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("user_avatar_id_fkey")
                    .from(User::Table, User::AvatarId)
                    .to(File::Table, File::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("user_banner_id_fkey")
                    .from(User::Table, User::BannerId)
                    .to(File::Table, File::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("file_user_id_fkey")
                    .from(File::Table, File::UserId)
                    .to(User::Table, User::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("file_post_id_fkey")
                    .from(File::Table, File::PostId)
                    .to(Post::Table, Post::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(File::Table)
                    .name("file_post_id_fkey")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(File::Table)
                    .name("file_user_id_fkey")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(User::Table)
                    .name("user_banner_id_fkey")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(User::Table)
                    .name("user_avatar_id_fkey")
                    .to_owned(),
            )
            .await?;

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
            .drop_table(Table::drop().table(File::Table).to_owned())
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
    UpdatedAt,
    LastFetchedAt,
    Handle,
    Name,
    FollowerCount,
    FollowingCount,
    PostCount,
    AvatarId,
    BannerId,
    IsBot,
    Host,
    Inbox,
    PublicKey,
    PrivateKey,
    Uri,
}

#[derive(Iden)]
enum File {
    Table,
    Id,
    CreatedAt,
    UserId,
    PostId,
    Hash,
    Mime,
    Path,
    Url,
    ThumbnailUrl,
    IsSensitive,
    OriginalUrl,
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
