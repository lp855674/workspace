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
