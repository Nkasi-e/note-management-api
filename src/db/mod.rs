use std::time::Duration;

use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::config::settings::AppConfig;

pub async fn init_pg_pool(config: &AppConfig) -> PgPool {
    PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .acquire_timeout(Duration::from_secs(config.database.connection_timeout))
        .connect(&config.database.database_url)
        .await
        .expect("Failed to create Postgres pool")
}


