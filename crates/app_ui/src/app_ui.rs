use assets::{load_default_theme, load_windows_keymap};
use dock::DockPlacement;
use gpui::{AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div};
use gpui_component::{
    TitleBar,
    tree::{TreeItem, TreeState},
};
use keymap::KeyBinding;
use status_bar::StatusBarItem;
use theme::ThemeDefinition;
use ui::{
    ShellDockHost, ShellSidebar, ShellStatus, ShellWorkspace, render_file_tree, render_shell,
};
use workspace::WorkspaceController;

pub struct AppFrame {
    title: &'static str,
    pub workspace: WorkspaceController,
    pub theme: ThemeDefinition,
    pub keymap: Vec<KeyBinding>,
    pub status_items: Vec<StatusBarItem>,
    file_tree: Option<Entity<TreeState>>,
}

impl AppFrame {
    pub const fn title() -> &'static str {
        "Zed Workbench Kernel"
    }

    pub fn new(workspace: WorkspaceController) -> Self {
        Self {
            title: Self::title(),
            workspace,
            theme: load_default_theme().unwrap_or_else(|_| ThemeDefinition::default()),
            keymap: load_windows_keymap().unwrap_or_default(),
            status_items: Vec::new(),
            file_tree: None,
        }
    }

    pub fn title_text(&self) -> &'static str {
        self.title
    }

    pub fn status_toolbar_text(&self) -> &'static str {
        ""
    }
}

impl Render for AppFrame {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let _ = window;
        if self.file_tree.is_none() {
            self.file_tree = Some(cx.new(|cx| TreeState::new(cx).items(Vec::<TreeItem>::new())));
        }

        let workspace_state = self.workspace.state();
        let focused_panel = workspace_state
            .focused_panel
            .as_deref()
            .unwrap_or("welcome.panel");
        let recent_commands = workspace_state.recent_commands.len();
        let center_panel = workspace_state
            .dock_layout
            .active_panel(DockPlacement::Center)
            .map(|_| focused_panel.to_owned())
            .or_else(|| workspace_state.focused_panel.clone());
        let right_panel = workspace_state
            .dock_layout
            .active_panel(DockPlacement::Right)
            .map(|_| focused_panel.to_owned());
        let bottom_panel = workspace_state
            .dock_layout
            .active_panel(DockPlacement::Bottom)
            .map(|_| focused_panel.to_owned());

        let sidebar = ShellSidebar {
            workspace_name: "workspace".to_owned(),
            project_root: "E:\\code\\workspace".to_owned(),
            tree: render_file_tree(
                self.file_tree
                    .as_ref()
                    .expect("file tree should be initialized before render"),
            )
            .into_any_element(),
        };
        let workspace = ShellWorkspace {
            center: ShellDockHost {
                id: "dock.center",
                title: "Center",
                active_panel: center_panel,
                visible: true,
            },
            right: ShellDockHost {
                id: "dock.right",
                title: "Right Dock",
                active_panel: right_panel,
                visible: workspace_state.dock_layout.is_visible(DockPlacement::Right),
            },
            bottom: ShellDockHost {
                id: "dock.bottom",
                title: "Bottom Dock",
                active_panel: bottom_panel,
                visible: workspace_state
                    .dock_layout
                    .is_visible(DockPlacement::Bottom),
            },
        };
        let status = ShellStatus {
            left_text: format!("focused: {focused_panel}"),
            right_text: format!("recent commands: {recent_commands}"),
            items: self.status_items.clone(),
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(
                TitleBar::new().child(
                    div()
                        .flex()
                        .items_center()
                        .text_sm()
                        .child(self.title),
                ),
            )
            .child(render_shell(sidebar, workspace, status))
    }
}
