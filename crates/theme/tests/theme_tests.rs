use theme::ThemeDefinition;

#[test]
fn theme_definition_default_has_expected_name_and_token() {
    let theme = ThemeDefinition::default();
    assert_eq!(theme.name, "Default");
    assert!(theme.tokens.contains_key("status_bar.background"));
}
