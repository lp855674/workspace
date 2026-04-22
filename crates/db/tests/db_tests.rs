use db::schema_sql;

#[test]
fn initial_schema_contains_workspace_sessions() {
    assert!(schema_sql().contains("create table workspace_sessions"));
}
