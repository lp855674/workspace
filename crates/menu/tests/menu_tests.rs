use commands::CommandDescriptor;
use menu::MenuItem;

#[test]
fn menu_item_uses_command_descriptor_values() {
    let command = CommandDescriptor::try_new("workspace.toggle_left_dock", "Toggle Left Dock")
        .expect("descriptor should be valid");
    let menu_item = MenuItem::from_command(command);

    assert_eq!(
        menu_item.command_id().as_str(),
        "workspace.toggle_left_dock"
    );
    assert_eq!(menu_item.title().as_str(), "Toggle Left Dock");
}
