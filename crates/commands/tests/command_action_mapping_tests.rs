use actions::{ActionEnvelope, PanelAction};
use commands::CommandDescriptor;

#[test]
fn command_descriptor_carries_a_typed_action() {
    let descriptor = CommandDescriptor::try_new_with_action(
        "welcome.open",
        "Open Welcome",
        ActionEnvelope::panel(PanelAction::Open {
            panel_type_id: "welcome.panel".to_owned(),
        }),
    )
    .expect("command descriptor should be valid");

    assert_eq!(descriptor.action().kind(), "panel");
}
