use foundation::{AppError, Id};

#[test]
fn id_new_empty_returns_invalid_id_error() {
    let result = Id::new("");
    assert_eq!(result, Err(AppError::InvalidId));
}

#[test]
fn id_new_valid_builds_and_display_preserves_value() {
    let id = Id::new("agent-42").expect("valid id should construct");
    assert_eq!(id.to_string(), "agent-42");
}
