use app_ui::AppFrame;
use gpui_component::TitleBar;

#[test]
fn app_title_matches_framework_goal() {
    assert_eq!(AppFrame::title(), "Zed Workbench Kernel");
}

#[test]
fn app_uses_gpui_component_title_bar_options() {
    let options = TitleBar::title_bar_options();

    assert!(options.appears_transparent);
    assert!(options.title.is_none());
    assert!(options.traffic_light_position.is_some());
}
