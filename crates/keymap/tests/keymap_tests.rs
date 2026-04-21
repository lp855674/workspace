use keymap::KeyBinding;

#[test]
fn key_binding_new_preserves_command_id() {
    let binding = KeyBinding::new("workspace.toggle_left_dock", "ctrl-b");
    assert_eq!(binding.command_id, "workspace.toggle_left_dock");
}
