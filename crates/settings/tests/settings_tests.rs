use settings::AppSettings;

#[test]
fn app_settings_default_restores_session_on_launch() {
    let settings = AppSettings::default();
    assert!(settings.restore_session_on_launch);
    assert_eq!(settings.theme_name, "Default");
}
