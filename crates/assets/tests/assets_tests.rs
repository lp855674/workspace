use assets::{load_default_theme, load_windows_keymap};

#[test]
fn bundled_default_theme_is_parseable() {
    let theme = load_default_theme().expect("default theme asset should parse");
    assert_eq!(theme.name, "Default");
    assert!(theme.tokens.contains_key("status_bar.background"));
}

#[test]
fn bundled_windows_keymap_is_parseable() {
    let keymap = load_windows_keymap().expect("default keymap asset should parse");
    assert!(!keymap.is_empty());
    assert_eq!(keymap[0].command_id, "workspace.toggle_left_dock");
}
