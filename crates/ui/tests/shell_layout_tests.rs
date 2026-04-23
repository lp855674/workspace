use status_bar::StatusBarItem;
use ui::{ShellDockHost, ShellStatusToolbar, ShellWorkspace};

#[test]
fn shell_workspace_models_bottom_dock_separately() {
    let workspace = ShellWorkspace {
        center: ShellDockHost::new(
            "dock.center",
            "Center",
            Some("welcome.panel".to_owned()),
            true,
        ),
        right: ShellDockHost::new("dock.right", "Right Dock", None, false),
        bottom: ShellDockHost::new(
            "dock.bottom",
            "Bottom Dock",
            Some("terminal.panel".to_owned()),
            true,
        ),
    };

    assert_eq!(workspace.bottom.id, "dock.bottom");
    assert!(workspace.bottom.visible);
    assert_eq!(
        workspace.bottom.active_panel.as_deref(),
        Some("terminal.panel")
    );
}

#[test]
fn status_toolbar_defaults_to_contribution_items_only() {
    let toolbar = ShellStatusToolbar {
        items: Vec::<StatusBarItem>::new(),
    };

    assert!(toolbar.items.is_empty());
}
