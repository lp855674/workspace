use theme::ThemeDefinition;

#[test]
fn theme_definition_default_has_expected_name_and_tokens() {
    let theme = ThemeDefinition::default();
    assert_eq!(theme.name, "Default");
    assert_eq!(theme.tokens.len(), 20);
    assert_eq!(
        theme.tokens.get("app.background"),
        Some(&"#0f1419".to_string())
    );
    assert_eq!(
        theme.tokens.get("sidebar.background"),
        Some(&"#121921".to_string())
    );
    assert_eq!(
        theme.tokens.get("status_bar.background"),
        Some(&"#111820".to_string())
    );
    assert_eq!(theme.token_hex("status_bar.background", 0), 0x111820);
    assert_eq!(theme.token_hex("missing", 0xabcdef), 0xabcdef);
}

#[test]
fn default_theme_json_matches_code_default() {
    let theme_from_asset: ThemeDefinition =
        serde_json::from_str(assets::DEFAULT_THEME_JSON).expect("valid theme json");

    assert_eq!(theme_from_asset, ThemeDefinition::default());
}
