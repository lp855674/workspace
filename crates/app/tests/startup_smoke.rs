use app_ui::AppFrame;

#[test]
fn app_title_matches_framework_goal() {
    assert_eq!(AppFrame::title(), "Zed Workbench Kernel");
}
