pub use sea_orm_migration::prelude::*;

mod m20230806_104639_initial;
mod m20230811_152513_reaction;
mod m20230811_163629_local_file;
mod m20230812_032603_emoji;
mod m20230812_061845_mention;
mod m20230812_071308_post_emoji;
mod m20230812_114735_hashtag;

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
        ]
    }
}
