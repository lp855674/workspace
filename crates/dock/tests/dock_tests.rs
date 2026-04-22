use dock::{DockPlacement, VisibleDockState};

#[test]
fn visible_dock_state_with_visible_has_single_panel() {
    let state = VisibleDockState::with_visible(DockPlacement::Left, "welcome.panel");

    assert_eq!(state.visible_panels.len(), 1);
}
