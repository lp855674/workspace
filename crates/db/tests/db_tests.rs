use db::initial_migration_sql;
use sqlx::{Connection, Row, SqliteConnection};
use std::collections::BTreeSet;

#[tokio::test]
async fn initial_migration_creates_expected_tables() {
    let mut connection = connect_and_migrate().await;
    let rows = sqlx::query("select name from sqlite_master where type = 'table'")
        .fetch_all(&mut connection)
        .await
        .expect("querying sqlite_master should work");

    let table_names: BTreeSet<String> = rows
        .into_iter()
        .map(|row| row.get::<String, _>("name"))
        .collect();

    let expected_tables = BTreeSet::from([
        "command_history".to_string(),
        "module_state".to_string(),
        "notifications".to_string(),
        "panel_states".to_string(),
        "workspace_sessions".to_string(),
    ]);

    assert!(expected_tables.is_subset(&table_names));
}

#[tokio::test]
async fn initial_migration_enforces_panel_states_and_command_history_constraints() {
    let mut connection = connect_and_migrate().await;

    sqlx::query("insert into workspace_sessions (session_id, created_at) values (?, ?)")
        .bind("session-1")
        .bind("2026-01-01T00:00:00Z")
        .execute(&mut connection)
        .await
        .expect("workspace_sessions insert should work");

    sqlx::query("insert into panel_states (session_id, panel_id, dock, visible) values (?, ?, ?, ?)")
        .bind("session-1")
        .bind("panel-1")
        .bind("left")
        .bind(1_i64)
        .execute(&mut connection)
        .await
        .expect("first panel_states insert should work");

    let duplicate_panel_state = sqlx::query(
        "insert into panel_states (session_id, panel_id, dock, visible) values (?, ?, ?, ?)",
    )
    .bind("session-1")
    .bind("panel-1")
    .bind("right")
    .bind(0_i64)
    .execute(&mut connection)
    .await;
    assert!(duplicate_panel_state.is_err());

    let missing_session_fk = sqlx::query(
        "insert into panel_states (session_id, panel_id, dock, visible) values (?, ?, ?, ?)",
    )
    .bind("missing-session")
    .bind("panel-2")
    .bind("left")
    .bind(1_i64)
    .execute(&mut connection)
    .await;
    assert!(missing_session_fk.is_err());

    sqlx::query("insert into command_history (command_id, created_at) values (?, ?)")
        .bind("command-1")
        .bind("2026-01-01T00:00:00Z")
        .execute(&mut connection)
        .await
        .expect("first command_history insert should work");

    let duplicate_command_id =
        sqlx::query("insert into command_history (command_id, created_at) values (?, ?)")
            .bind("command-1")
            .bind("2026-01-01T00:01:00Z")
            .execute(&mut connection)
            .await;
    assert!(duplicate_command_id.is_err());
}

async fn connect_and_migrate() -> SqliteConnection {
    let mut connection = SqliteConnection::connect("sqlite::memory:")
        .await
        .expect("in-memory sqlite should open");
    sqlx::query("pragma foreign_keys = on;")
        .execute(&mut connection)
        .await
        .expect("enabling foreign keys should work");
    sqlx::raw_sql(initial_migration_sql())
        .execute(&mut connection)
        .await
        .expect("initial migration should execute");
    connection
}
