/// SQL text for the first (initial) SQLite migration.
///
/// Keep this focused on the bootstrap schema in `migrations/0001_init.sql`.
pub fn initial_migration_sql() -> &'static str {
    include_str!("../migrations/0001_init.sql")
}

/// Backward-compatible alias used by existing plans/tests.
///
/// This returns the same SQL as [`initial_migration_sql`], not an aggregate of
/// all migrations.
pub fn schema_sql() -> &'static str {
    initial_migration_sql()
}
