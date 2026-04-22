use panel::{DockPlacement, PanelDescriptor};

#[test]
fn panel_descriptor_new_preserves_caller_values() {
    let descriptor = PanelDescriptor::new("welcome.panel", "Welcome", DockPlacement::Center);

    assert_eq!(descriptor.id, "welcome.panel");
    assert_eq!(descriptor.title, "Welcome");
    assert_eq!(descriptor.dock, DockPlacement::Center);
    assert!(descriptor.restorable);
}
