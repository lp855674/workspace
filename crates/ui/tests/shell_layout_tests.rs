use status_bar::StatusBarItem;
use ui::{
    ShellDockHost, ShellStatusToolbar, ShellTerminalCell, ShellTerminalSession, ShellTerminalTab,
    ShellWorkspace,
};

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
        sidebar_visible: true,
        right_visible: false,
        bottom_visible: false,
        local_shell_active: false,
    };

    assert!(toolbar.items.is_empty());
}

#[test]
fn dock_hosts_can_carry_a_real_terminal_session() {
    let host = ShellDockHost::new(
        "dock.bottom",
        "Bottom Dock",
        Some("Terminal".to_owned()),
        true,
    )
    .with_terminal_tabs(vec![
        ShellTerminalTab {
            id: "terminal-1".to_owned(),
            title: "Terminal 1".to_owned(),
            active: true,
        },
        ShellTerminalTab {
            id: "terminal-2".to_owned(),
            title: "Terminal 2".to_owned(),
            active: false,
        },
    ])
    .with_terminal(ui::ShellTerminalSession {
        shell_name: "pwsh.exe".to_owned(),
        cwd: "E:\\code\\workspace".to_owned(),
        status: "running".to_owned(),
        title: "Terminal".to_owned(),
        rows: 20,
        cols: 120,
        cursor_row: 0,
        cursor_col: 23,
        cursor_hidden: false,
        focused: true,
        scrollback: 0,
        away_from_bottom: false,
        visible_cells: vec![vec![ShellTerminalCell {
            text: "PS".to_owned(),
            foreground: Some(0xcdd8e2),
            background: None,
            bold: true,
            underline: false,
        }]],
        visible_lines: vec!["PS E:\\code\\workspace> ".to_owned()],
    });

    assert_eq!(host.active_panel.as_deref(), Some("Terminal"));
    assert_eq!(host.terminal_tabs.len(), 2);
    assert_eq!(
        host.terminal
            .as_ref()
            .map(|terminal| terminal.shell_name.as_str()),
        Some("pwsh.exe")
    );
}

#[test]
fn terminal_tabs_track_active_identity() {
    let tabs = vec![
        ShellTerminalTab {
            id: "terminal-1".to_owned(),
            title: "Terminal 1".to_owned(),
            active: false,
        },
        ShellTerminalTab {
            id: "terminal-2".to_owned(),
            title: "Terminal 2".to_owned(),
            active: true,
        },
    ];

    let host = ShellDockHost::new(
        "dock.bottom",
        "Bottom Dock",
        Some("Terminal".to_owned()),
        true,
    )
    .with_terminal_tabs(tabs.clone())
    .with_terminal(ShellTerminalSession {
        shell_name: "pwsh.exe".to_owned(),
        cwd: "E:\\code\\workspace".to_owned(),
        status: "running".to_owned(),
        title: "Terminal".to_owned(),
        rows: 20,
        cols: 120,
        cursor_row: 0,
        cursor_col: 0,
        cursor_hidden: false,
        focused: false,
        scrollback: 12,
        away_from_bottom: true,
        visible_cells: vec![],
        visible_lines: vec![],
    });

    assert_eq!(
        host.terminal_tabs
            .iter()
            .find(|tab| tab.active)
            .map(|tab| tab.id.as_str()),
        Some("terminal-2")
    );
}

#[test]
fn dock_hosts_track_resizable_dimensions() {
    let host = ShellDockHost::new("dock.right", "Right", None, true)
        .with_size(gpui::px(360.), gpui::px(180.));

    assert_eq!(host.width, gpui::px(360.));
    assert_eq!(host.height, gpui::px(180.));
}
