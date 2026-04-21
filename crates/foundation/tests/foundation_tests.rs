use foundation::{AppError, Id};

#[test]
fn id_new_empty_returns_invalid_id_error() {
    let result = Id::new("");
    assert_eq!(result, Err(AppError::InvalidId));
}
