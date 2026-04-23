use actions::{ActionEnvelope, PanelAction};
use dock::DockPlacement;
use panel::{PanelDescriptor, PanelTypeId};
use workspace::{SerializedPanelState, SerializedWorkspaceSession, WorkspaceController};

#[test]
fn toggle_panel_opens_then_hides_the_same_singleton_instance() {
    let mut workspace = WorkspaceController::new("session-1");
    workspace.register_panel(PanelDescriptor::singleton(
        PanelTypeId::try_new("welcome.panel").expect("panel type id should parse"),
        "Welcome",
        DockPlacement::Center,
    ));

    workspace.dispatch(ActionEnvelope::panel(PanelAction::Toggle {
        panel_type_id: "welcome.panel".to_owned(),
    }));
    assert!(workspace.state().is_panel_visible("welcome.panel"));
    let first_instance_id = workspace
        .state()
        .live_panels
        .get("welcome.panel")
        .expect("singleton runtime should exist")
        .instance_id
        .clone();

    workspace.dispatch(ActionEnvelope::panel(PanelAction::Toggle {
        panel_type_id: "welcome.panel".to_owned(),
    }));
    assert!(!workspace.state().is_panel_visible("welcome.panel"));
    assert!(
        !workspace
            .state()
            .dock_layout
            .is_visible(DockPlacement::Center)
    );
    assert_eq!(
        workspace
            .state()
            .dock_layout
            .active_panel(DockPlacement::Center),
        None
    );

    workspace.dispatch(ActionEnvelope::panel(PanelAction::Open {
        panel_type_id: "welcome.panel".to_owned(),
    }));
    assert!(workspace.state().is_panel_visible("welcome.panel"));
    assert_eq!(
        workspace
            .state()
            .live_panels
            .get("welcome.panel")
            .expect("singleton runtime should still exist")
            .instance_id,
        first_instance_id
    );
}

#[test]
fn restore_skips_unknown_panels_and_keeps_startup_alive() {
    let mut workspace = WorkspaceController::new("session-1");
    let session = SerializedWorkspaceSession {
        schema_version: 2,
        panels: vec![SerializedPanelState {
            panel_type_id: "missing.panel".to_owned(),
            panel_instance_key: "missing.panel".to_owned(),
            placement: DockPlacement::Right,
            visible: true,
            focused: true,
            panel_state_json: Some("{}".to_owned()),
        }],
    };

    workspace.restore_session(&session);

    assert!(workspace.state().visible_panels().is_empty());
    assert!(workspace.session().restored);
}

#[test]
fn restored_non_default_placement_survives_later_serialization() {
    let mut workspace = WorkspaceController::new("session-1");
    workspace.register_panel(PanelDescriptor::singleton(
        PanelTypeId::try_new("welcome.panel").expect("panel type id should parse"),
        "Welcome",
        DockPlacement::Center,
    ));

    workspace.restore_session(&SerializedWorkspaceSession {
        schema_version: 2,
        panels: vec![SerializedPanelState {
            panel_type_id: "welcome.panel".to_owned(),
            panel_instance_key: "welcome.panel".to_owned(),
            placement: DockPlacement::Right,
            visible: true,
            focused: true,
            panel_state_json: None,
        }],
    });

    let serialized = workspace.serialize_session();

    assert_eq!(serialized.panels.len(), 1);
    assert_eq!(serialized.panels[0].placement, DockPlacement::Right);
    assert!(
        workspace
            .state()
            .dock_layout
            .is_visible(DockPlacement::Right)
    );
}

#[test]
fn move_panel_action_moves_singleton_to_target_dock() {
    let mut workspace = WorkspaceController::new("session-1");
    workspace.register_panel(PanelDescriptor::singleton(
        PanelTypeId::try_new("welcome.panel").expect("panel type id should parse"),
        "Welcome",
        DockPlacement::Center,
    ));

    workspace.dispatch(ActionEnvelope::panel(PanelAction::Open {
        panel_type_id: "welcome.panel".to_owned(),
    }));
    workspace.dispatch(ActionEnvelope::panel(PanelAction::Move {
        panel_type_id: "welcome.panel".to_owned(),
        dock: "right".to_owned(),
    }));

    let runtime = workspace
        .state()
        .live_panels
        .get("welcome.panel")
        .expect("moved panel should stay live");

    assert_eq!(runtime.placement, DockPlacement::Right);
    assert!(
        !workspace
            .state()
            .dock_layout
            .is_visible(DockPlacement::Center)
    );
    assert!(
        workspace
            .state()
            .dock_layout
            .is_visible(DockPlacement::Right)
    );
    assert_eq!(
        workspace
            .state()
            .dock_layout
            .active_panel(DockPlacement::Right),
        Some(&runtime.instance_key)
    );
}

#[test]
fn unregistered_panel_show_and_focus_do_not_create_fake_runtime_state() {
    let mut workspace = WorkspaceController::new("session-1");

    workspace.show_panel("missing.panel");
    workspace.focus_panel("missing.panel");

    assert!(workspace.state().visible_panels().is_empty());
    assert!(workspace.state().focused_panel.is_none());
    assert!(workspace.state().live_panels.is_empty());
}
