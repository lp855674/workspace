use dock::DockPlacement;
use panel::PanelDescriptor;
use workspace::{
    SerializedDockPanel, SerializedWorkspaceState, WorkspaceController, WorkspaceState,
};

#[test]
fn workspace_state_remembers_focused_panel() {
    let mut state = WorkspaceState::default();
    state.focus_panel("welcome.panel");
    assert_eq!(state.focused_panel.as_deref(), Some("welcome.panel"));
}

#[test]
fn workspace_state_serializes_visible_dock_panels() {
    let mut state = WorkspaceState::default();
    state.register_panel(PanelDescriptor::new(
        "welcome.panel",
        "Welcome",
        DockPlacement::Center,
    ));
    state.focus_panel("welcome.panel");

    let serialized = state.serialize();

    assert_eq!(serialized.visible_panels, vec!["welcome.panel".to_string()]);
    assert_eq!(
        serialized.dock_panels,
        vec![SerializedDockPanel {
            panel_id: "welcome.panel".to_string(),
            placement: DockPlacement::Center,
        }]
    );
}

#[test]
fn workspace_controller_marks_restored_sessions() {
    let mut controller = WorkspaceController::new("session-1");
    controller.restore_session(&SerializedWorkspaceState {
        visible_panels: vec!["welcome.panel".to_string()],
        focused_panel: Some("welcome.panel".to_string()),
        dock_panels: vec![SerializedDockPanel {
            panel_id: "welcome.panel".to_string(),
            placement: DockPlacement::Center,
        }],
    });

    assert!(controller.session().restored);
    assert_eq!(
        controller.state().focused_panel.as_deref(),
        Some("welcome.panel")
    );
}

#[test]
fn workspace_controller_can_toggle_command_palette() {
    let mut controller = WorkspaceController::new("session-1");
    assert!(!controller.state().command_palette.is_open());
    controller.toggle_command_palette();
    assert!(controller.state().command_palette.is_open());
}

#[test]
fn workspace_controller_can_be_built_from_existing_state() {
    let mut state = WorkspaceState::default();
    state.focus_panel("welcome.panel");

    let controller = WorkspaceController::from_state("session-2", state);

    assert_eq!(controller.session().session_id, "session-2");
    assert_eq!(
        controller.state().focused_panel.as_deref(),
        Some("welcome.panel")
    );
}

#[test]
fn workspace_controller_serializes_current_state() {
    let mut controller = WorkspaceController::new("session-3");
    controller.register_panel(PanelDescriptor::new(
        "welcome.panel",
        "Welcome",
        DockPlacement::Center,
    ));
    controller.focus_panel("welcome.panel");

    let serialized = controller.serialize();

    assert_eq!(serialized.focused_panel.as_deref(), Some("welcome.panel"));
    assert_eq!(serialized.visible_panels, vec!["welcome.panel".to_string()]);
}
