use theme::ThemeDefinition;

#[test]
fn theme_definition_default_has_expected_name_and_tokens() {
    let theme = ThemeDefinition::default();
    assert_eq!(theme.name, "Default");
    assert_eq!(theme.tokens.len(), 3);
    assert_eq!(
        theme.tokens.get("app.background"),
        Some(&"#111111".to_string())
    );
    assert_eq!(
        theme.tokens.get("sidebar.background"),
        Some(&"#1a1a1a".to_string())
    );
    assert_eq!(
        theme.tokens.get("status_bar.background"),
        Some(&"#222222".to_string())
    );
}

#[test]
fn default_theme_json_matches_code_default() {
    let theme_from_asset: ThemeDefinition =
        serde_json::from_str(assets::DEFAULT_THEME_JSON).expect("valid theme json");

    assert_eq!(theme_from_asset, ThemeDefinition::default());
}
