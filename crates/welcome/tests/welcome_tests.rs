use module::FeatureModule;
use panel::DockPlacement;
use welcome::WelcomeModule;

#[test]
fn welcome_module_uses_stable_module_id() {
    let module = WelcomeModule::default();
    assert_eq!(module.module_id(), "welcome");
}

#[test]
fn welcome_module_registers_center_panel() {
    let module = WelcomeModule::default();
    assert_eq!(module.panel().id, "welcome.panel");
    assert_eq!(module.panel().dock, DockPlacement::Center);
}
