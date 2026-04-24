use gpui::{
    AnyElement, App, CursorStyle, FocusHandle, FontWeight, IntoElement, KeyDownEvent,
    ParentElement, Pixels, Point, Render, ScrollWheelEvent, StatefulInteractiveElement, Window,
    div, prelude::*, px, rgb,
};
use gpui_component::{
    Icon, IconName, Sizable,
    list::ListItem,
    scroll::ScrollableElement,
    tooltip::Tooltip,
    tree::{TreeEntry, tree},
};
use status_bar::StatusBarItem;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShellTheme {
    pub app_background: u32,
    pub title_bar_background: u32,
    pub title_bar_foreground: u32,
    pub title_bar_border: u32,
    pub window_control_foreground: u32,
    pub window_control_hover: u32,
    pub window_control_close_hover: u32,
    pub sidebar_background: u32,
    pub sidebar_foreground: u32,
    pub sidebar_muted: u32,
    pub dock_background: u32,
    pub dock_header: u32,
    pub border: u32,
    pub resize_handle_hover: u32,
    pub status_bar_background: u32,
    pub status_bar_foreground: u32,
    pub status_bar_active: u32,
    pub status_bar_button: u32,
    pub status_bar_button_active: u32,
    pub status_bar_button_hover: u32,
}

impl Default for ShellTheme {
    fn default() -> Self {
        Self {
            app_background: 0x0f1419,
            title_bar_background: 0x10161d,
            title_bar_foreground: 0xe1e8ef,
            title_bar_border: 0x1f2a33,
            window_control_foreground: 0xf2f7fb,
            window_control_hover: 0x2b3844,
            window_control_close_hover: 0xc42b1c,
            sidebar_background: 0x121921,
            sidebar_foreground: 0xcdd8e2,
            sidebar_muted: 0x8092a2,
            dock_background: 0x0f1419,
            dock_header: 0x10161d,
            border: 0x1c2730,
            resize_handle_hover: 0x4d8fd6,
            status_bar_background: 0x111820,
            status_bar_foreground: 0x8fa2b4,
            status_bar_active: 0xdce7f0,
            status_bar_button: 0x141d26,
            status_bar_button_active: 0x1b2834,
            status_bar_button_hover: 0x24313d,
        }
    }
}

pub struct ShellSidebar {
    pub workspace_name: String,
    pub project_root: String,
    pub tree: AnyElement,
    pub visible: bool,
    pub width: Pixels,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShellTerminalTab {
    pub id: String,
    pub title: String,
    pub active: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShellTerminalCell {
    pub text: String,
    pub foreground: Option<u32>,
    pub background: Option<u32>,
    pub bold: bool,
    pub underline: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShellTerminalSession {
    pub shell_name: String,
    pub cwd: String,
    pub status: String,
    pub title: String,
    pub rows: u16,
    pub cols: u16,
    pub cursor_row: u16,
    pub cursor_col: u16,
    pub cursor_hidden: bool,
    pub focused: bool,
    pub scrollback: usize,
    pub away_from_bottom: bool,
    pub visible_cells: Vec<Vec<ShellTerminalCell>>,
    pub visible_lines: Vec<String>,
}

pub struct ShellDockHost {
    pub id: &'static str,
    pub title: &'static str,
    pub active_panel: Option<String>,
    pub terminal_tabs: Vec<ShellTerminalTab>,
    pub terminal: Option<ShellTerminalSession>,
    pub body: Option<AnyElement>,
    pub focus_handle: Option<FocusHandle>,
    pub on_key_down: Option<Rc<dyn Fn(&KeyDownEvent, &mut Window, &mut App)>>,
    pub on_scroll_wheel: Option<Rc<dyn Fn(&ScrollWheelEvent, &mut Window, &mut App)>>,
    pub visible: bool,
    pub width: Pixels,
    pub height: Pixels,
}

impl ShellDockHost {
    pub fn new(
        id: &'static str,
        title: &'static str,
        active_panel: Option<String>,
        visible: bool,
    ) -> Self {
        Self {
            id,
            title,
            active_panel,
            terminal_tabs: Vec::new(),
            terminal: None,
            body: None,
            focus_handle: None,
            on_key_down: None,
            on_scroll_wheel: None,
            visible,
            width: px(320.),
            height: px(200.),
        }
    }

    pub fn with_terminal(mut self, terminal: ShellTerminalSession) -> Self {
        self.terminal = Some(terminal);
        self
    }

    pub fn with_terminal_tabs(mut self, terminal_tabs: Vec<ShellTerminalTab>) -> Self {
        self.terminal_tabs = terminal_tabs;
        self
    }

    pub fn with_body(mut self, body: AnyElement) -> Self {
        self.body = Some(body);
        self
    }

    pub fn with_terminal_interaction(
        mut self,
        focus_handle: FocusHandle,
        on_key_down: Rc<dyn Fn(&KeyDownEvent, &mut Window, &mut App)>,
        on_scroll_wheel: Rc<dyn Fn(&ScrollWheelEvent, &mut Window, &mut App)>,
    ) -> Self {
        self.focus_handle = Some(focus_handle);
        self.on_key_down = Some(on_key_down);
        self.on_scroll_wheel = Some(on_scroll_wheel);
        self
    }

    pub fn with_size(mut self, width: Pixels, height: Pixels) -> Self {
        self.width = width;
        self.height = height;
        self
    }
}

pub struct ShellWorkspace {
    pub center: ShellDockHost,
    pub right: ShellDockHost,
    pub bottom: ShellDockHost,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShellStatusToolbar {
    pub items: Vec<StatusBarItem>,
    pub sidebar_visible: bool,
    pub right_visible: bool,
    pub bottom_visible: bool,
    pub local_shell_active: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShellResizeTarget {
    Left,
    Right,
    Bottom,
}

#[derive(Clone, Debug)]
pub enum ShellAction {
    ToggleSidebar,
    ToggleRightDock,
    ToggleBottomDock,
    OpenLocalShell,
    SpawnTerminal,
    CloseTerminal,
    ActivateTerminalTab(String),
    CloseTerminalTab(String),
    ScrollTerminalViewport(i32),
    Resize(ShellResizeTarget, Point<Pixels>),
}

pub struct ShellInteractions {
    pub on_action: Rc<dyn Fn(&ShellAction, &mut Window, &mut App)>,
}

pub fn render_shell(
    sidebar: ShellSidebar,
    workspace: ShellWorkspace,
    status: ShellStatusToolbar,
    interactions: ShellInteractions,
    theme: ShellTheme,
) -> impl IntoElement {
    let status_interactions = ShellInteractions {
        on_action: interactions.on_action.clone(),
    };
    let resize_interactions = ShellInteractions {
        on_action: interactions.on_action.clone(),
    };

    div()
        .size_full()
        .flex()
        .flex_col()
        .bg(rgb(theme.app_background))
        .text_color(gpui::white())
        .child(
            div()
                .flex()
                .flex_1()
                .min_h_0()
                .when(sidebar.visible, |this| {
                    this.child(render_sidebar(sidebar, theme))
                        .child(render_resize_handle(ShellResizeTarget::Left, theme))
                })
                .child(render_workspace_body(workspace, interactions, theme)),
        )
        .child(render_status_toolbar(status, status_interactions, theme))
        .on_drag_move::<ShellResizeTarget>(move |event, window, cx| {
            if event.event.dragging() {
                (resize_interactions.on_action)(
                    &ShellAction::Resize(*event.drag(cx), event.event.position),
                    window,
                    cx,
                );
            }
        })
}

fn render_sidebar(sidebar: ShellSidebar, theme: ShellTheme) -> impl IntoElement {
    div()
        .w(sidebar.width)
        .h_full()
        .flex()
        .flex_col()
        .bg(rgb(theme.sidebar_background))
        .border_r_1()
        .border_color(rgb(theme.border))
        .child(
            div()
                .w_full()
                .px_3()
                .pt_3()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(theme.sidebar_foreground))
                        .child(sidebar.workspace_name),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(theme.sidebar_muted))
                        .child(sidebar.project_root),
                ),
        )
        .child(div().flex_1().min_h_0().px_2().py_2().child(sidebar.tree))
        .child(
            div()
                .w_full()
                .px_3()
                .pb_3()
                .flex()
                .justify_between()
                .items_center()
                .text_xs()
                .text_color(rgb(theme.sidebar_muted))
                .child("Explorer"),
        )
}

fn render_workspace_body(
    workspace: ShellWorkspace,
    interactions: ShellInteractions,
    theme: ShellTheme,
) -> impl IntoElement {
    let center = workspace.center;
    let right = workspace.right;
    let bottom = workspace.bottom;
    let right_visible = right.visible;
    let bottom_visible = bottom.visible;

    div().flex_1().min_w_0().min_h_0().flex().child(
        div()
            .flex()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .child(render_dock_host(
                        center,
                        DockHostKind::Center,
                        &interactions,
                        theme,
                    ))
                    .when(bottom_visible, |this| {
                        this.child(render_resize_handle(ShellResizeTarget::Bottom, theme))
                            .child(render_dock_host(
                                bottom,
                                DockHostKind::Bottom,
                                &interactions,
                                theme,
                            ))
                    }),
            )
            .when(right_visible, |this| {
                this.child(render_resize_handle(ShellResizeTarget::Right, theme))
                    .child(render_dock_host(
                        right,
                        DockHostKind::Right,
                        &interactions,
                        theme,
                    ))
            }),
    )
}

#[derive(Clone, Copy)]
enum DockHostKind {
    Center,
    Right,
    Bottom,
}

fn render_dock_host(
    host: ShellDockHost,
    kind: DockHostKind,
    interactions: &ShellInteractions,
    theme: ShellTheme,
) -> impl IntoElement {
    let active_panel = host.active_panel.clone();
    let terminal_tabs = host.terminal_tabs.clone();
    let terminal = host.terminal.clone();
    let body = host.body;
    let focus_handle = host.focus_handle.clone();
    let on_key_down = host.on_key_down.clone();
    let on_scroll_wheel = host.on_scroll_wheel.clone();
    let is_terminal = terminal.is_some();
    let mut container = div()
        .bg(rgb(theme.dock_background))
        .border_color(rgb(theme.border))
        .flex()
        .flex_col()
        .when_some(active_panel.clone(), |this, panel| {
            this.child(render_dock_header(
                panel,
                terminal_tabs,
                is_terminal,
                terminal.clone(),
                interactions,
                theme,
            ))
        })
        .child(render_panel_surface(
            host.active_panel,
            terminal,
            body,
            focus_handle,
            on_key_down,
            on_scroll_wheel,
            theme,
        ));

    container = match kind {
        DockHostKind::Center => container.flex_1().min_w_0().min_h_0(),
        DockHostKind::Right => container.w(host.width).h_full().border_l_1(),
        DockHostKind::Bottom => container.h(host.height).border_t_1(),
    };

    container
}

fn render_dock_header(
    panel: String,
    terminal_tabs: Vec<ShellTerminalTab>,
    is_terminal: bool,
    terminal: Option<ShellTerminalSession>,
    interactions: &ShellInteractions,
    theme: ShellTheme,
) -> impl IntoElement {
    let header = div()
        .h(px(30.))
        .px_1p5()
        .flex()
        .items_center()
        .justify_between()
        .border_b_1()
        .border_color(rgb(theme.border))
        .bg(rgb(theme.dock_header));

    if is_terminal {
        header
            .child(render_terminal_tab_strip(
                terminal_tabs,
                interactions,
                theme,
            ))
            .child(render_terminal_header_actions(
                terminal,
                interactions,
                theme,
            ))
    } else {
        header.child(
            div()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(theme.title_bar_foreground))
                .child(panel),
        )
    }
}

fn render_panel_surface(
    active_panel: Option<String>,
    terminal: Option<ShellTerminalSession>,
    body: Option<AnyElement>,
    focus_handle: Option<FocusHandle>,
    on_key_down: Option<Rc<dyn Fn(&KeyDownEvent, &mut Window, &mut App)>>,
    on_scroll_wheel: Option<Rc<dyn Fn(&ScrollWheelEvent, &mut Window, &mut App)>>,
    theme: ShellTheme,
) -> impl IntoElement {
    let has_terminal = terminal.is_some();

    let panel = div()
        .flex_1()
        .min_h_0()
        .relative()
        .text_sm()
        .text_color(rgb(theme.sidebar_foreground))
        .when(active_panel.is_none(), |this| {
            this.bg(rgb(theme.dock_background))
        })
        .when_some(terminal, |this, terminal| {
            this.child(render_terminal_session(terminal, theme))
        })
        .when_some(body, |this, body| {
            if has_terminal {
                this.child(body)
            } else {
                this.child(body)
            }
        });

    if let (Some(focus_handle), Some(on_key_down), Some(on_scroll_wheel)) =
        (focus_handle, on_key_down, on_scroll_wheel)
    {
        let focus_handle_for_click = focus_handle.clone();
        panel
            .id("terminal-panel-surface")
            .track_focus(&focus_handle)
            .on_click(move |_, window, _| {
                window.focus(&focus_handle_for_click);
            })
            .on_key_down(move |event, window, cx| {
                on_key_down(event, window, cx);
            })
            .on_scroll_wheel(move |event, window, cx| {
                on_scroll_wheel(event, window, cx);
            })
            .into_any_element()
    } else {
        panel.into_any_element()
    }
}

fn render_terminal_session(terminal: ShellTerminalSession, theme: ShellTheme) -> impl IntoElement {
    let output_lines = terminal_output_lines(&terminal, theme);

    div().size_full().bg(rgb(theme.dock_background)).child(
        div()
            .size_full()
            .overflow_y_scrollbar()
            .font_family("Consolas")
            .text_xs()
            .line_height(px(18.))
            .text_color(rgb(theme.sidebar_foreground))
            .px_3()
            .py_2()
            .pr_5()
            .child(div().w_full().flex().flex_col().children(output_lines)),
    )
}

fn terminal_output_lines(terminal: &ShellTerminalSession, theme: ShellTheme) -> Vec<AnyElement> {
    if terminal.visible_cells.is_empty() {
        return vec![
            div()
                .text_color(rgb(theme.sidebar_muted))
                .child("Terminal ready. Focus the panel and type.")
                .into_any_element(),
        ];
    }

    terminal
        .visible_cells
        .iter()
        .enumerate()
        .map(|(row, cells)| {
            div()
                .w_full()
                .flex()
                .items_start()
                .whitespace_nowrap()
                .children(render_terminal_line(terminal, row as u16, cells, theme))
                .into_any_element()
        })
        .collect()
}

fn render_terminal_line(
    terminal: &ShellTerminalSession,
    row: u16,
    cells: &[ShellTerminalCell],
    theme: ShellTheme,
) -> Vec<AnyElement> {
    let max_col = cells.len().max(terminal.cursor_col as usize + 1);

    (0..max_col)
        .map(|col| {
            let mut cell = cells
                .get(col)
                .cloned()
                .unwrap_or_else(|| blank_terminal_cell());

            if terminal.focused
                && !terminal.cursor_hidden
                && terminal.cursor_row == row
                && terminal.cursor_col as usize == col
            {
                cell = terminal_cursor_cell(cell, theme);
            }

            render_terminal_cell(cell, theme).into_any_element()
        })
        .collect()
}

fn render_terminal_cell(cell: ShellTerminalCell, theme: ShellTheme) -> impl IntoElement {
    let text = preserve_terminal_cell_text(&cell.text);
    let foreground = cell.foreground.unwrap_or(theme.sidebar_foreground);
    let background = cell.background.unwrap_or(theme.dock_background);

    div()
        .flex_none()
        .min_w(px(0.))
        .text_xs()
        .text_color(rgb(foreground))
        .text_bg(rgb(background))
        .when(cell.bold, |this| this.font_weight(FontWeight::BOLD))
        .when(cell.underline, |this| this.underline())
        .child(text)
}

fn blank_terminal_cell() -> ShellTerminalCell {
    ShellTerminalCell {
        text: " ".to_owned(),
        foreground: None,
        background: None,
        bold: false,
        underline: false,
    }
}

fn terminal_cursor_cell(mut cell: ShellTerminalCell, theme: ShellTheme) -> ShellTerminalCell {
    let cursor_foreground = cell.background.unwrap_or(theme.dock_background);
    let cursor_background = cell.foreground.unwrap_or(theme.status_bar_active);
    cell.foreground = Some(cursor_foreground);
    cell.background = Some(cursor_background);
    cell
}

fn preserve_terminal_cell_text(text: &str) -> String {
    let rendered = if text.is_empty() { " " } else { text };
    rendered
        .chars()
        .map(|character| {
            if character == ' ' {
                '\u{00A0}'
            } else {
                character
            }
        })
        .collect()
}

fn render_status_toolbar(
    status: ShellStatusToolbar,
    interactions: ShellInteractions,
    theme: ShellTheme,
) -> impl IntoElement {
    let sidebar_interactions = interactions.on_action.clone();
    let right_interactions = interactions.on_action.clone();
    let shell_interactions = interactions.on_action.clone();

    div()
        .h(px(30.))
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .bg(rgb(theme.status_bar_background))
        .border_t_1()
        .border_color(rgb(theme.border))
        .child(
            div()
                .h_full()
                .flex()
                .items_center()
                .px_3()
                .gap_1()
                .border_r_1()
                .border_color(rgb(theme.border))
                .child(toolbar_button(
                    "status-toggle-sidebar",
                    if status.sidebar_visible {
                        IconName::PanelLeftOpen
                    } else {
                        IconName::PanelLeftClose
                    },
                    "Toggle Explorer",
                    status.sidebar_visible,
                    ShellAction::ToggleSidebar,
                    sidebar_interactions,
                    theme,
                    None,
                )),
        )
        .child(
            div()
                .h_full()
                .flex()
                .flex_1()
                .items_center()
                .px_3()
                .gap_1()
                .children(status.items.into_iter().map(|item| {
                    toolbar_static_icon("status-item", IconName::Info, item.text, theme)
                })),
        )
        .child(
            div()
                .h_full()
                .flex()
                .items_center()
                .px_2()
                .gap_1()
                .border_l_1()
                .border_color(rgb(theme.border))
                .child(toolbar_button(
                    "status-open-local-shell",
                    IconName::SquareTerminal,
                    "Open Local Shell",
                    status.local_shell_active,
                    ShellAction::OpenLocalShell,
                    shell_interactions,
                    theme,
                    Some("pwsh"),
                ))
                .child(toolbar_button(
                    "status-toggle-right",
                    if status.right_visible {
                        IconName::PanelRightOpen
                    } else {
                        IconName::PanelRightClose
                    },
                    "Toggle Right Dock",
                    status.right_visible,
                    ShellAction::ToggleRightDock,
                    right_interactions,
                    theme,
                    None,
                )),
        )
}

fn render_terminal_header_actions(
    terminal: Option<ShellTerminalSession>,
    interactions: &ShellInteractions,
    theme: ShellTheme,
) -> impl IntoElement {
    let jump_interactions = interactions.on_action.clone();
    let spawn_interactions = interactions.on_action.clone();
    let close_interactions = interactions.on_action.clone();

    div()
        .flex()
        .items_center()
        .gap_1()
        .when_some(
            terminal.filter(|terminal| terminal.away_from_bottom),
            |this, terminal| {
                this.child(
                    div()
                        .h(px(22.))
                        .px_2()
                        .flex()
                        .items_center()
                        .gap_1()
                        .rounded(px(4.))
                        .border_1()
                        .border_color(rgb(theme.border))
                        .bg(rgb(theme.status_bar_button_active))
                        .text_color(rgb(theme.status_bar_active))
                        .child(
                            div()
                                .text_xs()
                                .child(format!("{} lines up", terminal.scrollback)),
                        )
                        .child(header_button(
                            "terminal-jump-bottom",
                            IconName::ChevronDown,
                            "Jump To Bottom",
                            ShellAction::ScrollTerminalViewport(i32::MIN),
                            jump_interactions,
                            theme,
                        )),
                )
            },
        )
        .child(header_button(
            "terminal-new",
            IconName::Plus,
            "New Terminal",
            ShellAction::SpawnTerminal,
            spawn_interactions,
            theme,
        ))
        .child(header_button(
            "terminal-close",
            IconName::Close,
            "Close Terminal",
            ShellAction::CloseTerminal,
            close_interactions,
            theme,
        ))
}

fn render_terminal_tab_strip(
    terminal_tabs: Vec<ShellTerminalTab>,
    interactions: &ShellInteractions,
    theme: ShellTheme,
) -> impl IntoElement {
    div().flex_1().min_w_0().overflow_x_scrollbar().child(
        div()
            .flex()
            .flex_none()
            .items_center()
            .gap_0p5()
            .pr_2()
            .children(
                terminal_tabs
                    .into_iter()
                    .enumerate()
                    .map(|(index, tab)| render_terminal_tab(index, tab, interactions, theme)),
            ),
    )
}

fn render_terminal_tab(
    index: usize,
    tab: ShellTerminalTab,
    interactions: &ShellInteractions,
    theme: ShellTheme,
) -> impl IntoElement {
    let activate_interactions = interactions.on_action.clone();
    let close_interactions = interactions.on_action.clone();
    let tab_id = tab.id.clone();
    let close_tab_id = tab.id.clone();

    div()
        .id(("terminal-tab", index))
        .h(px(22.))
        .min_w(px(112.))
        .max_w(px(220.))
        .px_2()
        .flex()
        .flex_shrink_0()
        .items_center()
        .gap_1()
        .rounded_t(px(4.))
        .border_1()
        .border_color(if tab.active {
            rgb(theme.status_bar_active)
        } else {
            rgb(theme.dock_header)
        })
        .bg(if tab.active {
            rgb(theme.status_bar_button)
        } else {
            rgb(theme.dock_header)
        })
        .text_color(if tab.active {
            rgb(theme.status_bar_active)
        } else {
            rgb(theme.status_bar_foreground)
        })
        .hover(move |style| {
            style
                .bg(rgb(theme.status_bar_button_hover))
                .text_color(rgb(theme.status_bar_active))
        })
        .cursor(CursorStyle::PointingHand)
        .on_click(move |_, window, cx| {
            activate_interactions(
                &ShellAction::ActivateTerminalTab(tab_id.clone()),
                window,
                cx,
            );
        })
        .child(Icon::new(IconName::SquareTerminal).xsmall())
        .child(
            div()
                .min_w_0()
                .flex_1()
                .overflow_hidden()
                .text_xs()
                .truncate()
                .child(tab.title),
        )
        .child(
            div()
                .id(("terminal-tab-close", index))
                .h(px(16.))
                .w(px(16.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(3.))
                .text_color(if tab.active {
                    rgb(theme.status_bar_active)
                } else {
                    rgb(theme.status_bar_foreground)
                })
                .hover(move |style| {
                    style
                        .bg(rgb(theme.status_bar_button_hover))
                        .text_color(rgb(theme.status_bar_active))
                })
                .on_click(move |_, window, cx| {
                    close_interactions(
                        &ShellAction::CloseTerminalTab(close_tab_id.clone()),
                        window,
                        cx,
                    );
                })
                .child(Icon::new(IconName::Close).xsmall()),
        )
}

fn header_button(
    id: &'static str,
    icon: IconName,
    tooltip: &'static str,
    action: ShellAction,
    on_action: Rc<dyn Fn(&ShellAction, &mut Window, &mut App)>,
    theme: ShellTheme,
) -> impl IntoElement {
    div()
        .id(id)
        .h(px(22.))
        .w(px(22.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(4.))
        .text_color(rgb(theme.status_bar_foreground))
        .hover(move |style| {
            style
                .bg(rgb(theme.status_bar_button_hover))
                .text_color(rgb(theme.status_bar_active))
        })
        .cursor(CursorStyle::PointingHand)
        .tooltip(move |window, cx| Tooltip::new(tooltip).build(window, cx))
        .on_click(move |_, window, cx| {
            on_action(&action, window, cx);
        })
        .child(Icon::new(icon).xsmall())
}

fn toolbar_button(
    id: &'static str,
    icon: IconName,
    tooltip: &'static str,
    active: bool,
    action: ShellAction,
    on_action: Rc<dyn Fn(&ShellAction, &mut Window, &mut App)>,
    theme: ShellTheme,
    label: Option<&'static str>,
) -> impl IntoElement {
    div()
        .id(id)
        .h(px(24.))
        .min_w(if label.is_some() { px(54.) } else { px(30.) })
        .px_2()
        .flex()
        .items_center()
        .justify_center()
        .gap_1()
        .rounded(px(4.))
        .border_1()
        .border_color(if active {
            rgb(theme.border)
        } else {
            rgb(theme.status_bar_background)
        })
        .text_color(if active {
            rgb(theme.status_bar_active)
        } else {
            rgb(theme.status_bar_foreground)
        })
        .bg(if active {
            rgb(theme.status_bar_button_active)
        } else {
            rgb(theme.status_bar_button)
        })
        .hover(move |style| {
            style
                .bg(rgb(theme.status_bar_button_hover))
                .text_color(rgb(theme.status_bar_active))
        })
        .cursor(CursorStyle::PointingHand)
        .tooltip(move |window, cx| Tooltip::new(tooltip).build(window, cx))
        .on_click(move |_, window, cx| {
            on_action(&action, window, cx);
        })
        .child(Icon::new(icon).small())
        .when_some(label, |this, label| {
            this.child(div().text_xs().child(label))
        })
}

fn toolbar_static_icon(
    id: &'static str,
    icon: IconName,
    tooltip: impl Into<String>,
    theme: ShellTheme,
) -> impl IntoElement {
    let tooltip = tooltip.into();
    div()
        .id(id)
        .h(px(24.))
        .w(px(28.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(4.))
        .text_color(rgb(theme.status_bar_foreground))
        .tooltip(move |window, cx| Tooltip::new(tooltip.clone()).build(window, cx))
        .child(Icon::new(icon).small())
}

fn render_resize_handle(target: ShellResizeTarget, theme: ShellTheme) -> impl IntoElement {
    let (id, horizontal) = match target {
        ShellResizeTarget::Left => ("resize-left", true),
        ShellResizeTarget::Right => ("resize-right", true),
        ShellResizeTarget::Bottom => ("resize-bottom", false),
    };

    div()
        .id(id)
        .flex_shrink_0()
        .bg(rgb(theme.border))
        .hover(move |style| style.bg(rgb(theme.resize_handle_hover)))
        .when(horizontal, |this| {
            this.w(px(1.)).h_full().cursor(CursorStyle::ResizeLeftRight)
        })
        .when(!horizontal, |this| {
            this.h(px(1.)).w_full().cursor(CursorStyle::ResizeUpDown)
        })
        .on_drag(target, |_, _, _, cx| cx.new(|_| ResizeDragPreview))
}

struct ResizeDragPreview;

impl Render for ResizeDragPreview {
    fn render(&mut self, _window: &mut Window, _cx: &mut gpui::Context<Self>) -> impl IntoElement {
        div().w(px(0.)).h(px(0.))
    }
}

pub fn render_file_tree(
    tree_state: &gpui::Entity<gpui_component::tree::TreeState>,
) -> impl IntoElement {
    tree(tree_state, |index, entry: &TreeEntry, selected, _, _| {
        let item = entry.item();
        let icon = if entry.is_folder() {
            if entry.is_expanded() {
                Icon::new(IconName::FolderOpen)
            } else {
                Icon::new(IconName::FolderClosed)
            }
        } else {
            Icon::new(IconName::File)
        };

        ListItem::new(index).selected(selected).px_2().py_1().child(
            div()
                .pl(px(12.0) * entry.depth() as f32)
                .flex()
                .items_center()
                .gap_2()
                .child(icon.small().text_color(rgb(0x8aa0b4)))
                .child(
                    div()
                        .text_sm()
                        .text_color(if item.is_disabled() {
                            rgb(0x556775)
                        } else {
                            rgb(0xcdd8e2)
                        })
                        .child(item.label.clone()),
                ),
        )
    })
}
