use keymap::KeyBinding;

#[test]
fn key_binding_new_preserves_command_id() {
    let binding = KeyBinding::new("workspace.toggle_left_dock", "ctrl-b");
    assert_eq!(binding.command_id, "workspace.toggle_left_dock");
    assert_eq!(binding.keystroke, "ctrl-b");
}

#[test]
fn default_windows_keymap_json_deserializes() {
    let bindings: Vec<KeyBinding> =
        serde_json::from_str(assets::DEFAULT_WINDOWS_KEYMAP_JSON).expect("valid keymap json");

    assert_eq!(bindings.len(), 2);
    assert_eq!(bindings[0].command_id, "workspace.toggle_left_dock");
    assert_eq!(bindings[0].keystroke, "ctrl-b");
}
