use actions::ActionDescriptor;

#[test]
fn action_descriptor_rejects_empty_ids() {
    assert!(ActionDescriptor::try_new("   ").is_err());
}

#[test]
fn action_descriptor_normalizes_trimmed_ids() {
    let action = ActionDescriptor::try_new(" workspace.toggle_left_dock ").unwrap();
    assert_eq!(action.id(), "workspace.toggle_left_dock");
}
