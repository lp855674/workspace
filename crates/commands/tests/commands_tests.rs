use commands::{CommandDescriptor, CommandRegistry};

#[test]
fn registry_rejects_duplicate_command_ids() {
    let mut registry = CommandRegistry::default();
    let command = CommandDescriptor::new("workspace.toggle_left_dock", "Toggle Left Dock");

    let first_result = registry.register(command.clone());
    assert!(first_result.is_ok());

    let duplicate_result = registry.register(command);
    assert!(duplicate_result.is_err());
}
