//! Database connection pool

use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

pub type DbPool = PgPool;

/// Initialize the database connection pool
pub async fn init_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(20)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(600))
        .connect(database_url)
        .await
}

/// Run database migrations
pub async fn run_migrations(pool: &DbPool) -> Result<(), sqlx::Error> {
    // The init-db.sql is run by Docker on startup
    // This is for any runtime migrations if needed
    tracing::info!("Database pool initialized successfully");
    Ok(())
}
