use module::ModuleRuntime;
use panel::PanelMultiplicity;
use welcome::WelcomeModule;
use welcome::WelcomePanelState;

#[test]
fn welcome_module_registers_a_singleton_panel_type_and_open_action() {
    let mut runtime = ModuleRuntime::default();
    runtime
        .register(Box::new(WelcomeModule::default()))
        .expect("welcome module should register");

    let registered = runtime
        .retained_modules()
        .first()
        .expect("welcome module should be retained");

    assert_eq!(registered.panels()[0].id.as_str(), "welcome.panel");
    assert_eq!(
        registered.panels()[0].multiplicity(),
        PanelMultiplicity::Singleton
    );
    assert_eq!(registered.commands()[0].id().as_str(), "welcome.open");
    assert_eq!(registered.commands()[0].action().kind(), "panel");
}

#[test]
fn welcome_panel_state_round_trips_with_defaults() {
    let state = WelcomePanelState::default();
    let json = serde_json::to_string(&state).expect("welcome panel state should serialize");
    let restored: WelcomePanelState =
        serde_json::from_str(&json).expect("welcome panel state should deserialize");

    assert_eq!(state, restored);
}
