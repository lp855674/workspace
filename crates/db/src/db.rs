use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Connection, SqliteConnection, SqlitePool};
use std::str::FromStr;

/// SQL text for the first (initial) SQLite migration.
///
/// Keep this focused on the bootstrap schema in `migrations/0001_init.sql`.
pub fn initial_migration_sql() -> &'static str {
    include_str!("../migrations/0001_init.sql")
}

/// Parse SQLite connection options with foreign key enforcement enabled.
pub fn sqlite_connect_options(database_url: &str) -> Result<SqliteConnectOptions, sqlx::Error> {
    Ok(SqliteConnectOptions::from_str(database_url)?.foreign_keys(true))
}

/// Open a SQLite connection with foreign key enforcement enabled.
pub async fn connect_sqlite(database_url: &str) -> Result<SqliteConnection, sqlx::Error> {
    SqliteConnection::connect_with(&sqlite_connect_options(database_url)?).await
}

/// Open a SQLite pool with foreign key enforcement enabled.
pub async fn connect_sqlite_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    SqlitePoolOptions::new()
        .connect_with(sqlite_connect_options(database_url)?)
        .await
}
