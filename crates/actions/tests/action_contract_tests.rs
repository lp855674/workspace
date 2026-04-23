use actions::{ActionEnvelope, PanelAction};

#[test]
fn panel_action_envelope_serializes_a_welcome_toggle() {
    let envelope = ActionEnvelope::panel(PanelAction::Toggle {
        panel_type_id: "welcome.panel".to_owned(),
    });

    let json = serde_json::to_string(&envelope).expect("action envelope should serialize");

    assert_eq!(
        json,
        "{\"kind\":\"panel\",\"action\":\"toggle\",\"panel_type_id\":\"welcome.panel\"}",
    );
}
