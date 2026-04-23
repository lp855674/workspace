use panel::{
    DockPlacement, PanelCloseBehavior, PanelDescriptor, PanelInstanceId, PanelInstanceKey,
    PanelMultiplicity, PanelTypeId,
};

#[test]
fn singleton_panel_descriptor_uses_type_id_as_default_instance_key() {
    let descriptor = PanelDescriptor::singleton(
        PanelTypeId::try_new("welcome.panel").expect("panel type id should parse"),
        "Welcome",
        DockPlacement::Center,
    );

    assert_eq!(descriptor.multiplicity(), PanelMultiplicity::Singleton);
    assert_eq!(descriptor.close_behavior(), PanelCloseBehavior::Hide);
    assert_eq!(
        descriptor.default_instance_key(),
        &PanelInstanceKey::from_static("welcome.panel"),
    );
}

#[test]
fn panel_instance_id_is_distinct_from_panel_type_id() {
    let panel_type_id = PanelTypeId::try_new("explorer.panel").expect("panel type id should parse");
    let panel_instance_id = PanelInstanceId::new("explorer.panel#1");

    assert_eq!(panel_type_id.as_str(), "explorer.panel");
    assert_eq!(panel_instance_id.as_str(), "explorer.panel#1");
    assert_ne!(panel_type_id.as_str(), panel_instance_id.as_str());
}

#[test]
#[should_panic(expected = "panel descriptor id must not be empty")]
fn panel_descriptor_new_rejects_empty_ids() {
    let _ = PanelDescriptor::new("   ", "Welcome", DockPlacement::Center);
}
