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
    if sqlite_pool_url_is_unsafe_in_memory(database_url) {
        return Err(sqlx::Error::Configuration(
            "pooled sqlite::memory: URLs are unsafe; use a shared-cache memory URI or a file-backed database".into(),
        ));
    }

    SqlitePoolOptions::new()
        .connect_with(sqlite_connect_options(database_url)?)
        .await
}

fn sqlite_pool_url_is_unsafe_in_memory(database_url: &str) -> bool {
    let normalized_url = database_url
        .strip_prefix("sqlite://")
        .or_else(|| database_url.strip_prefix("sqlite:"))
        .unwrap_or(database_url);
    let mut database_and_params = normalized_url.splitn(2, '?');
    let database = database_and_params.next().unwrap_or_default();
    let params = database_and_params.next().unwrap_or_default();

    if database == ":memory:" {
        return true;
    }

    let mut mode_is_memory = false;
    let mut cache_is_private = false;

    for pair in params.split('&').filter(|pair| !pair.is_empty()) {
        let mut key_and_value = pair.splitn(2, '=');
        let key = key_and_value.next().unwrap_or_default();
        let value = key_and_value.next().unwrap_or_default();

        match (key, value) {
            ("mode", "memory") => mode_is_memory = true,
            ("cache", "private") => cache_is_private = true,
            _ => {}
        }
    }

    mode_is_memory && cache_is_private
}
