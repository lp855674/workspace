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
async fn initial_migration_enforces_panel_states_command_history_and_notifications_constraints() {
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
        .bind("Left")
        .bind(1_i64)
        .execute(&mut connection)
        .await
        .expect("first panel_states insert should work");

    let duplicate_panel_state = sqlx::query(
        "insert into panel_states (session_id, panel_id, dock, visible) values (?, ?, ?, ?)",
    )
    .bind("session-1")
    .bind("panel-1")
    .bind("Right")
    .bind(0_i64)
    .execute(&mut connection)
    .await;
    assert!(duplicate_panel_state.is_err());

    let missing_session_fk = sqlx::query(
        "insert into panel_states (session_id, panel_id, dock, visible) values (?, ?, ?, ?)",
    )
    .bind("missing-session")
    .bind("panel-2")
    .bind("Left")
    .bind(1_i64)
    .execute(&mut connection)
    .await;
    assert!(missing_session_fk.is_err());

    let invalid_dock = sqlx::query(
        "insert into panel_states (session_id, panel_id, dock, visible) values (?, ?, ?, ?)",
    )
    .bind("session-1")
    .bind("panel-invalid-dock")
    .bind("left")
    .bind(1_i64)
    .execute(&mut connection)
    .await;
    assert!(invalid_dock.is_err());

    let invalid_visible = sqlx::query(
        "insert into panel_states (session_id, panel_id, dock, visible) values (?, ?, ?, ?)",
    )
    .bind("session-1")
    .bind("panel-invalid-visible")
    .bind("Center")
    .bind(2_i64)
    .execute(&mut connection)
    .await;
    assert!(invalid_visible.is_err());

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

    // Keep this list in sync with the schema's explicit closed set.
    let allowed_levels = ["info", "warning", "error"];
    for level in allowed_levels {
        sqlx::query(
            "insert into notifications (notification_id, level, message, created_at) values (?, ?, ?, ?)",
        )
        .bind(format!("notification-{level}"))
        .bind(level)
        .bind("ok")
        .bind("2026-01-01T00:00:00Z")
        .execute(&mut connection)
        .await
        .expect("allowed notification level insert should work");
    }

    let invalid_notification_level = sqlx::query(
        "insert into notifications (notification_id, level, message, created_at) values (?, ?, ?, ?)",
    )
    .bind("notification-invalid")
    .bind("debug")
    .bind("bad")
    .bind("2026-01-01T00:00:00Z")
    .execute(&mut connection)
    .await;
    assert!(invalid_notification_level.is_err());
}

#[tokio::test]
async fn initial_migration_enforces_module_state_json_validity_when_available() {
    let mut connection = connect_and_migrate().await;

    sqlx::query(
        "insert into module_state (module_id, state_key, state_json, updated_at) values (?, ?, ?, ?)",
    )
    .bind("module-1")
    .bind("state-1")
    .bind("{\"enabled\":true}")
    .bind("2026-01-01T00:00:00Z")
    .execute(&mut connection)
    .await
    .expect("valid module_state JSON should insert");

    let json_functions_available = sqlx::query_scalar::<_, i64>("select json_valid('{}');")
        .fetch_one(&mut connection)
        .await
        .is_ok();

    if json_functions_available {
        let invalid_json = sqlx::query(
            "insert into module_state (module_id, state_key, state_json, updated_at) values (?, ?, ?, ?)",
        )
        .bind("module-1")
        .bind("state-invalid")
        .bind("{invalid-json}")
        .bind("2026-01-01T00:01:00Z")
        .execute(&mut connection)
        .await;
        assert!(invalid_json.is_err());
    }
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
