use dock::{DockLayoutState, DockPlacement};
use panel::PanelInstanceKey;

#[test]
fn opening_a_panel_sets_the_active_tab_for_its_dock() {
    let mut layout = DockLayoutState::default();
    let key = PanelInstanceKey::from_static("welcome.panel");

    layout.show_panel(DockPlacement::Center, key.clone());

    assert_eq!(layout.active_panel(DockPlacement::Center), Some(&key));
    assert!(layout.is_visible(DockPlacement::Center));
}

#[test]
fn moving_a_panel_removes_it_from_the_old_dock_and_activates_it_in_the_new_dock() {
    let mut layout = DockLayoutState::default();
    let key = PanelInstanceKey::from_static("welcome.panel");

    layout.show_panel(DockPlacement::Center, key.clone());
    layout.move_panel(&key, DockPlacement::Right);

    assert_eq!(layout.active_panel(DockPlacement::Center), None);
    assert_eq!(layout.active_panel(DockPlacement::Right), Some(&key));
}
