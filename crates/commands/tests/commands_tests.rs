use commands::{
    CommandDescriptor, CommandDescriptorError, CommandId, CommandRegistry, RegisterCommandError,
};

#[test]
fn registry_rejects_duplicate_command_ids() {
    let mut registry = CommandRegistry::default();
    let command = CommandDescriptor::try_new("workspace.toggle_left_dock", "Toggle Left Dock")
        .expect("descriptor should be valid");

    let first_result = registry.register(command.clone());
    assert!(first_result.is_ok());

    let duplicate_result = registry.register(command);
    assert_eq!(
        duplicate_result,
        Err(RegisterCommandError::DuplicateCommandId(
            CommandId::try_new("workspace.toggle_left_dock").expect("id should be valid"),
        )),
    );
}

#[test]
fn descriptor_rejects_empty_command_id() {
    let result = CommandDescriptor::try_new("   ", "Toggle Left Dock");
    assert_eq!(result, Err(CommandDescriptorError::EmptyId));
}

#[test]
fn descriptor_rejects_whitespace_in_command_id() {
    let result = CommandDescriptor::try_new("workspace.toggle left_dock", "Toggle Left Dock");
    assert_eq!(result, Err(CommandDescriptorError::IdContainsWhitespace));
}

#[test]
fn descriptor_rejects_empty_title() {
    let result = CommandDescriptor::try_new("workspace.toggle_left_dock", "    ");
    assert_eq!(result, Err(CommandDescriptorError::EmptyTitle));
}
