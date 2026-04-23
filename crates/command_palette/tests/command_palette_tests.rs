use command_palette::CommandPaletteState;
use commands::CommandDescriptor;

#[test]
fn palette_toggle_flips_open_state() {
    let mut state = CommandPaletteState::default();
    assert!(!state.is_open());

    state.toggle();
    assert!(state.is_open());

    state.toggle();
    assert!(!state.is_open());
}

#[test]
fn palette_filters_commands_by_query() {
    let commands = vec![
        CommandDescriptor::try_new("workspace.toggle_left_dock", "Toggle Left Dock").unwrap(),
        CommandDescriptor::try_new("welcome.open", "Open Welcome").unwrap(),
    ];

    let mut state = CommandPaletteState::default();
    state.set_query("welcome");

    let matches = state.matching_commands(&commands);
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].id().as_str(), "welcome.open");
}
