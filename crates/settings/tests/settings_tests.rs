use settings::AppSettings;

#[test]
fn app_settings_default_restores_session_on_launch() {
    let settings = AppSettings::default();
    assert!(settings.restore_session_on_launch);
    assert_eq!(settings.theme_name, "Default");
}

#[test]
fn app_settings_partial_json_uses_defaults_for_missing_fields() {
    let settings: AppSettings =
        serde_json::from_str(r#"{ "theme_name": "Solarized" }"#).expect("valid settings json");

    assert!(settings.restore_session_on_launch);
    assert_eq!(settings.theme_name, "Solarized");
}
