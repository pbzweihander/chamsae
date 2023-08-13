pub use sea_orm_migration::prelude::*;

mod m20230806_104639_initial;
mod m20230811_152513_reaction;
mod m20230811_163629_local_file;
mod m20230812_032603_emoji;
mod m20230812_061845_mention;
mod m20230812_071308_post_emoji;
mod m20230812_114735_hashtag;
mod m20230812_123019_avatar_and_banner;
mod m20230812_130217_shared_inbox;
mod m20230812_132921_manually_approves_followers;
mod m20230812_134054_is_bot;
mod m20230812_135017_setting;
mod m20230812_155209_nodeinfo;
mod m20230812_161529_user_description;
mod m20230813_051423_report;
mod m20230813_081932_object_store_type;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230806_104639_initial::Migration),
            Box::new(m20230811_152513_reaction::Migration),
            Box::new(m20230811_163629_local_file::Migration),
            Box::new(m20230812_032603_emoji::Migration),
            Box::new(m20230812_061845_mention::Migration),
            Box::new(m20230812_071308_post_emoji::Migration),
            Box::new(m20230812_114735_hashtag::Migration),
            Box::new(m20230812_123019_avatar_and_banner::Migration),
            Box::new(m20230812_130217_shared_inbox::Migration),
            Box::new(m20230812_132921_manually_approves_followers::Migration),
            Box::new(m20230812_134054_is_bot::Migration),
            Box::new(m20230812_135017_setting::Migration),
            Box::new(m20230812_155209_nodeinfo::Migration),
            Box::new(m20230812_161529_user_description::Migration),
            Box::new(m20230813_051423_report::Migration),
            Box::new(m20230813_081932_object_store_type::Migration),
        ]
    }
}
