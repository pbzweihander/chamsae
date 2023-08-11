use std::sync::Arc;

use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct State {
    pub db: Arc<DatabaseConnection>,
}