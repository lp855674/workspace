#[path = "../src/main.rs"]
mod app_main;

use app_ui::AppFrame;
use workspace::WorkspaceController;

#[test]
fn boot_message_references_the_workbench_kernel_shell() {
    assert_eq!(app_main::boot_message(), "starting Zed Workbench Kernel");
}

#[test]
fn app_frame_uses_workbench_kernel_title() {
    let frame = AppFrame::new(WorkspaceController::new("session-1"));

    assert_eq!(AppFrame::title(), "Zed Workbench Kernel");
    assert_eq!(frame.title_text(), "Zed Workbench Kernel");
}

#[test]
fn app_frame_starts_without_fake_shell_status_items() {
    let frame = AppFrame::new(WorkspaceController::new("session-1"));

    assert!(frame.status_items.is_empty());
}

#[test]
fn app_frame_starts_with_empty_status_toolbar_text() {
    let frame = AppFrame::new(WorkspaceController::new("session-1"));

    assert_eq!(frame.status_toolbar_text(), "");
}
