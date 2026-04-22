use dock::DockPlacement;
use panel::PanelDescriptor;

#[test]
fn panel_descriptor_new_defaults_to_restorable() {
    let descriptor = PanelDescriptor::new("welcome.panel", "Welcome", DockPlacement::Center);

    assert!(descriptor.restorable);
}
