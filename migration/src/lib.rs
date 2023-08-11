pub use sea_orm_migration::prelude::*;

mod m20230806_104639_initial;
mod m20230811_152513_reaction;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230806_104639_initial::Migration),
            Box::new(m20230811_152513_reaction::Migration),
        ]
    }
}
