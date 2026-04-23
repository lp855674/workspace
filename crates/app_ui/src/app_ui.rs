use assets::{load_default_theme, load_windows_keymap};
use dock::DockPlacement;
use gpui::{
    AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render, Styled,
    Window, WindowControlArea, div, px, rgb,
};
use gpui_component::{
    Icon, IconName, Sizable,
    tree::{TreeItem, TreeState},
};
use keymap::KeyBinding;
use status_bar::StatusBarItem;
use theme::ThemeDefinition;
use ui::{
    ShellDockHost, ShellSidebar, ShellStatusToolbar, ShellWorkspace, render_file_tree, render_shell,
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
            center: ShellDockHost::new("dock.center", "Center", center_panel, true),
            right: ShellDockHost::new(
                "dock.right",
                "Right Dock",
                right_panel,
                workspace_state.dock_layout.is_visible(DockPlacement::Right),
            ),
            bottom: ShellDockHost::new(
                "dock.bottom",
                "Bottom Dock",
                bottom_panel,
                workspace_state
                    .dock_layout
                    .is_visible(DockPlacement::Bottom),
            ),
        };
        let status = ShellStatusToolbar {
            items: self.status_items.clone(),
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(render_title_bar(self.title, window))
            .child(render_shell(sidebar, workspace, status))
    }
}

fn render_title_bar(title: &'static str, window: &Window) -> impl IntoElement {
    div()
        .h(px(36.))
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .bg(rgb(0x10161d))
        .border_b_1()
        .border_color(rgb(0x1f2a33))
        .child(
            div()
                .h_full()
                .flex()
                .items_center()
                .flex_1()
                .px_3()
                .text_sm()
                .text_color(rgb(0xe1e8ef))
                .window_control_area(WindowControlArea::Drag)
                .child(title),
        )
        .child(
            div()
                .h_full()
                .flex()
                .items_center()
                .child(window_control_button(
                    "window-minimize",
                    IconName::WindowMinimize,
                    WindowControlArea::Min,
                ))
                .child(window_control_button(
                    "window-maximize",
                    if window.is_maximized() {
                        IconName::WindowRestore
                    } else {
                        IconName::WindowMaximize
                    },
                    WindowControlArea::Max,
                ))
                .child(window_control_button(
                    "window-close",
                    IconName::WindowClose,
                    WindowControlArea::Close,
                )),
        )
}

fn window_control_button(
    id: &'static str,
    icon: IconName,
    control_area: WindowControlArea,
) -> impl IntoElement {
    let is_close = matches!(control_area, WindowControlArea::Close);
    div()
        .id(id)
        .h_full()
        .w(px(48.))
        .flex()
        .items_center()
        .justify_center()
        .text_color(rgb(0xc8d2dc))
        .hover(move |style| {
            if is_close {
                style.bg(rgb(0xc42b1c)).text_color(rgb(0xffffff))
            } else {
                style.bg(rgb(0x26323d)).text_color(rgb(0xffffff))
            }
        })
        .window_control_area(control_area)
        .child(Icon::new(icon).small())
}
