use assets::{load_default_theme, load_windows_keymap};
use dock::DockPlacement;
use gpui::{
    AppContext, Context, Entity, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent,
    ParentElement, Render, ScrollWheelEvent, Styled, Timer, Window, WindowControlArea, div, px,
    rgb,
};
use gpui_component::{
    IconName,
    tree::{TreeItem, TreeState},
};
use keymap::KeyBinding;
use status_bar::StatusBarItem;
use std::cell::Cell;
use std::path::Path;
use std::rc::Rc;
use std::time::Duration;
use terminal::{TERMINAL_PANEL_ID, TerminalCell, TerminalSession};
use theme::ThemeDefinition;
use ui::{
    ShellAction, ShellDockHost, ShellInteractions, ShellResizeTarget, ShellSidebar,
    ShellStatusToolbar, ShellTerminalSession, ShellTerminalTab, ShellTheme, ShellWorkspace,
    render_file_tree, render_shell, titlebar_icon_button, titlebar_label, titlebar_tab_chip,
};
use workspace::WorkspaceController;

struct TerminalTabRuntime {
    id: String,
    label: String,
    session: TerminalSession,
    display_offset: usize,
    follow_output: bool,
    pending_display_offset: Rc<Cell<Option<usize>>>,
}

#[derive(Clone)]
struct TitleBarTab {
    label: String,
    active: bool,
}

const TERMINAL_HEADER_HEIGHT: f32 = 30.;
const TERMINAL_LINE_HEIGHT: f32 = 18.;
const TERMINAL_VERTICAL_PADDING: f32 = 16.;
const MIN_TERMINAL_ROWS: usize = 8;

pub struct AppFrame {
    title: &'static str,
    pub workspace: WorkspaceController,
    pub theme: ThemeDefinition,
    pub keymap: Vec<KeyBinding>,
    pub status_items: Vec<StatusBarItem>,
    file_tree: Option<Entity<TreeState>>,
    sidebar_visible: bool,
    right_dock_visible: bool,
    bottom_dock_visible: bool,
    sidebar_width: gpui::Pixels,
    right_dock_width: gpui::Pixels,
    bottom_dock_height: gpui::Pixels,
    terminal_tabs: Vec<TerminalTabRuntime>,
    active_terminal_id: Option<String>,
    next_terminal_id: usize,
    terminal_focus_handle: Option<FocusHandle>,
    terminal_poll_task: Option<gpui::Task<()>>,
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
            sidebar_visible: true,
            right_dock_visible: false,
            bottom_dock_visible: false,
            sidebar_width: px(255.),
            right_dock_width: px(320.),
            bottom_dock_height: px(200.),
            terminal_tabs: Vec::new(),
            active_terminal_id: None,
            next_terminal_id: 1,
            terminal_focus_handle: None,
            terminal_poll_task: None,
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
        if self.file_tree.is_none() {
            self.file_tree = Some(cx.new(|cx| TreeState::new(cx).items(Vec::<TreeItem>::new())));
        }
        if self.terminal_focus_handle.is_none() {
            self.terminal_focus_handle = Some(cx.focus_handle().tab_stop(true));
        }
        self.refresh_local_shell_status();

        let shell_theme = shell_theme_from_definition(&self.theme);
        let workspace_state = self.workspace.state();
        let bottom_panel_id = active_panel_id(workspace_state, DockPlacement::Bottom);
        let center_panel = active_panel_title(workspace_state, DockPlacement::Center)
            .or_else(|| panel_title(workspace_state, workspace_state.focused_panel.as_deref()));
        let right_panel = active_panel_title(workspace_state, DockPlacement::Right);
        let bottom_panel = active_panel_title(workspace_state, DockPlacement::Bottom);
        let terminal_session = self.active_terminal().map(|tab| {
            let snapshot = tab.session.snapshot();
            ShellTerminalSession {
                shell_name: snapshot.shell_name,
                cwd: snapshot.cwd,
                status: snapshot.status,
                title: snapshot.title,
                rows: snapshot.rows,
                cols: snapshot.cols,
                cursor_row: snapshot.cursor_row,
                cursor_col: snapshot.cursor_col,
                cursor_hidden: snapshot.cursor_hidden,
                focused: self
                    .terminal_focus_handle
                    .as_ref()
                    .is_some_and(|focus_handle| focus_handle.is_focused(window)),
                scrollback: snapshot.scrollback,
                viewport_top: snapshot.viewport_top,
                viewport_height: snapshot.viewport_height,
                total_lines: snapshot.total_lines,
                can_scroll_up: snapshot.can_scroll_up,
                can_scroll_down: snapshot.can_scroll_down,
                away_from_bottom: snapshot.can_scroll_down,
                scrollbar_line_height: px(TERMINAL_LINE_HEIGHT),
                pending_display_offset: tab.pending_display_offset.clone(),
                visible_cells: snapshot
                    .visible_cells
                    .into_iter()
                    .map(|row| row.into_iter().map(shell_terminal_cell).collect())
                    .collect(),
                visible_lines: snapshot.visible_lines,
            }
        });
        let terminal_tabs = self
            .terminal_tabs
            .iter()
            .map(|tab| {
                let title = terminal_tab_title(tab);
                ShellTerminalTab {
                    id: tab.id.clone(),
                    title,
                    active: self.active_terminal_id.as_deref() == Some(tab.id.as_str()),
                }
            })
            .collect::<Vec<_>>();
        let workspace_meta = workspace_meta();
        let title_bar_tabs = build_title_bar_tabs(
            center_panel.as_deref(),
            right_panel.as_deref(),
            bottom_panel.as_deref(),
        );

        let sidebar = ShellSidebar {
            workspace_name: workspace_meta.name.clone(),
            project_root: workspace_meta.root.clone(),
            visible: self.sidebar_visible,
            width: self.sidebar_width,
            tree: self
                .file_tree
                .as_ref()
                .map(|tree| render_file_tree(tree).into_any_element())
                .unwrap_or_else(|| div().into_any_element()),
        };
        let workspace = ShellWorkspace {
            center: ShellDockHost::new("dock.center", "Center", center_panel, true),
            right: ShellDockHost::new(
                "dock.right",
                "Right Dock",
                right_panel,
                self.right_dock_visible
                    || workspace_state.dock_layout.is_visible(DockPlacement::Right),
            )
            .with_size(self.right_dock_width, px(0.)),
            bottom: ShellDockHost::new(
                "dock.bottom",
                "Bottom Dock",
                bottom_panel,
                self.bottom_dock_visible
                    || workspace_state
                        .dock_layout
                        .is_visible(DockPlacement::Bottom),
            )
            .with_size(px(0.), self.bottom_dock_height),
        };
        let workspace = if bottom_panel_id.as_deref() == Some(TERMINAL_PANEL_ID) {
            let mut bottom_host = workspace.bottom;
            if let Some(terminal) = terminal_session {
                bottom_host = bottom_host.with_terminal(terminal);
            }
            bottom_host = bottom_host.with_terminal_tabs(terminal_tabs);
            if let Some(focus_handle) = self.terminal_focus_handle.as_ref() {
                bottom_host = bottom_host.with_terminal_interaction(
                    focus_handle.clone(),
                    Rc::new(cx.listener(|frame, event: &KeyDownEvent, _window, cx| {
                        frame.handle_terminal_key_event(event, cx);
                    })),
                    Rc::new(cx.listener(|frame, event: &ScrollWheelEvent, _window, cx| {
                        frame.handle_terminal_scroll_wheel(event, cx);
                    })),
                );
            }
            ShellWorkspace {
                bottom: bottom_host,
                ..workspace
            }
        } else {
            workspace
        };
        let status = ShellStatusToolbar {
            items: self.status_items.clone(),
            sidebar_visible: self.sidebar_visible,
            right_visible: workspace.right.visible,
            bottom_visible: workspace.bottom.visible,
            local_shell_active: !self.terminal_tabs.is_empty(),
        };
        let interactions = ShellInteractions {
            on_action: Rc::new(cx.listener(|frame, action: &ShellAction, window, cx| {
                frame.handle_shell_action(action, window, cx);
            })),
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(render_platform_title_bar(
                self.title,
                &workspace_meta,
                &title_bar_tabs,
                window,
                shell_theme,
            ))
            .child(render_shell(
                sidebar,
                workspace,
                status,
                interactions,
                shell_theme,
            ))
    }
}

impl AppFrame {
    fn handle_shell_action(
        &mut self,
        action: &ShellAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match action {
            ShellAction::ToggleSidebar => {
                self.sidebar_visible = !self.sidebar_visible;
            }
            ShellAction::ToggleRightDock => {
                self.right_dock_visible = !self.right_dock_visible;
            }
            ShellAction::ToggleBottomDock => {
                self.bottom_dock_visible = !self.bottom_dock_visible;
                if self.bottom_dock_visible {
                    self.workspace.focus_panel(TERMINAL_PANEL_ID);
                    self.focus_terminal_surface(window);
                } else {
                    self.workspace.hide_panel(TERMINAL_PANEL_ID);
                }
            }
            ShellAction::OpenLocalShell => {
                self.bottom_dock_visible = true;
                self.workspace.focus_panel(TERMINAL_PANEL_ID);
                self.ensure_terminal_session(cx);
                self.focus_terminal_surface(window);
            }
            ShellAction::SpawnTerminal => {
                self.bottom_dock_visible = true;
                self.workspace.focus_panel(TERMINAL_PANEL_ID);
                self.spawn_terminal_session(cx);
                self.focus_terminal_surface(window);
            }
            ShellAction::CloseTerminal => {
                self.close_active_terminal();
            }
            ShellAction::ActivateTerminalTab(tab_id) => {
                if self.activate_terminal(tab_id) {
                    self.bottom_dock_visible = true;
                    self.workspace.focus_panel(TERMINAL_PANEL_ID);
                    self.focus_terminal_surface(window);
                }
            }
            ShellAction::CloseTerminalTab(tab_id) => {
                self.close_terminal_by_id(tab_id);
            }
            ShellAction::ScrollTerminalViewport(lines) => {
                self.scroll_active_terminal(*lines);
            }
            ShellAction::Resize(target, position) => {
                let bounds = window.window_bounds().get_bounds();
                match target {
                    ShellResizeTarget::Left => {
                        self.sidebar_width = clamp_px(position.x, 180., 420.);
                    }
                    ShellResizeTarget::Right => {
                        self.right_dock_width =
                            clamp_px(bounds.size.width - position.x, 240., 560.);
                    }
                    ShellResizeTarget::Bottom => {
                        self.bottom_dock_height =
                            clamp_px(bounds.size.height - position.y - px(30.), 120., 420.);
                    }
                }
            }
        }
        cx.notify();
    }

    fn upsert_status_item(&mut self, id: &str, text: String) {
        if let Some(item) = self.status_items.iter_mut().find(|item| item.id == id) {
            item.text = text;
            return;
        }

        self.status_items.push(StatusBarItem {
            id: id.to_owned(),
            text,
        });
    }

    fn refresh_local_shell_status(&mut self) {
        for tab in &mut self.terminal_tabs {
            if let Some(new_display_offset) = tab.pending_display_offset.take() {
                tab.display_offset = new_display_offset.min(tab.session.max_safe_scrollback());
                tab.follow_output = tab.display_offset == 0;
                tab.session.set_scrollback(tab.display_offset);
            }
            tab.session.refresh();
            if tab.follow_output {
                tab.session.set_scrollback(0);
            }
            tab.display_offset = tab.session.scrollback();
            tab.follow_output = tab.display_offset == 0;
        }
    }

    fn ensure_terminal_session(&mut self, cx: &mut Context<Self>) {
        let active_running = self
            .active_terminal_mut()
            .is_some_and(|tab| tab.session.is_running());

        if active_running {
            return;
        }

        if self.active_terminal_id.is_some() {
            self.close_active_terminal();
        }

        self.spawn_terminal_session(cx);
    }

    fn handle_terminal_key_event(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        if let Some(lines) =
            terminal_viewport_lines_for_key_event(event, self.visible_terminal_rows())
        {
            self.scroll_active_terminal(lines);
            cx.notify();
            return;
        }

        let Some(tab) = self.active_terminal_mut() else {
            self.upsert_status_item("local-shell", "Open a local shell first".to_owned());
            return;
        };
        tab.follow_output = true;
        tab.display_offset = 0;
        tab.session.set_scrollback(0);
        let Some(bytes) = key_event_to_bytes_with_mode(event, tab.session.application_cursor())
        else {
            return;
        };
        if let Err(message) = tab.session.write_bytes(&bytes) {
            self.upsert_status_item("local-shell", message);
        }
        cx.notify();
    }

    fn handle_terminal_scroll_wheel(&mut self, event: &ScrollWheelEvent, cx: &mut Context<Self>) {
        let lines = pixels_to_terminal_lines(event.delta.pixel_delta(px(18.)).y);
        if lines == 0 {
            return;
        }
        self.scroll_active_terminal(lines);
        cx.notify();
    }

    fn focus_terminal_surface(&self, window: &mut Window) {
        if let Some(focus_handle) = self.terminal_focus_handle.as_ref() {
            window.focus(focus_handle);
        }
    }

    fn resize_terminal_if_needed(&mut self) {
        let rows = terminal_rows_for_bottom_dock_height(self.bottom_dock_height) as u16;
        let cols = 120;
        for tab in &mut self.terminal_tabs {
            let _result = tab.session.resize(rows, cols);
            if tab.follow_output {
                tab.session.set_scrollback(0);
            } else {
                tab.session.set_scrollback(tab.display_offset);
            }
            tab.display_offset = tab.session.scrollback();
            tab.follow_output = tab.display_offset == 0;
        }
    }

    fn start_terminal_poll(&mut self, cx: &mut Context<Self>) {
        if self.terminal_poll_task.is_some() {
            return;
        }

        self.terminal_poll_task = Some(cx.spawn(async move |frame, cx| {
            loop {
                Timer::after(Duration::from_millis(100)).await;
                let keep_polling = frame
                    .update(cx, |frame, cx| {
                        frame.refresh_local_shell_status();
                        frame.resize_terminal_if_needed();
                        cx.notify();
                        !frame.terminal_tabs.is_empty()
                    })
                    .unwrap_or(false);
                if !keep_polling {
                    break;
                }
            }
        }));
    }

    fn spawn_terminal_session(&mut self, cx: &mut Context<Self>) {
        match TerminalSession::spawn_local() {
            Ok(session) => {
                let id = format!("terminal-{}", self.next_terminal_id);
                let label = format!("Terminal {}", self.next_terminal_id);
                let pending_display_offset = Rc::new(Cell::new(None));
                self.next_terminal_id += 1;
                self.terminal_tabs.push(TerminalTabRuntime {
                    id: id.clone(),
                    label,
                    session,
                    display_offset: 0,
                    follow_output: true,
                    pending_display_offset,
                });
                self.active_terminal_id = Some(id);
                if let Some(tab) = self.active_terminal_mut() {
                    tab.session.set_scrollback(0);
                    tab.display_offset = 0;
                    tab.follow_output = true;
                }
                self.start_terminal_poll(cx);
            }
            Err(message) => {
                self.upsert_status_item("local-shell", message);
            }
        }
    }

    fn active_terminal(&self) -> Option<&TerminalTabRuntime> {
        let active_id = self.active_terminal_id.as_deref()?;
        self.terminal_tabs
            .iter()
            .find(|tab| tab.id.as_str() == active_id)
    }

    fn active_terminal_mut(&mut self) -> Option<&mut TerminalTabRuntime> {
        let active_id = self.active_terminal_id.as_deref()?;
        self.terminal_tabs
            .iter_mut()
            .find(|tab| tab.id.as_str() == active_id)
    }

    fn visible_terminal_rows(&self) -> usize {
        terminal_rows_for_bottom_dock_height(self.bottom_dock_height)
    }

    fn activate_terminal(&mut self, tab_id: &str) -> bool {
        let exists = self
            .terminal_tabs
            .iter()
            .any(|tab| tab.id.as_str() == tab_id);
        if exists {
            self.active_terminal_id = Some(tab_id.to_owned());
            if let Some(tab) = self.active_terminal_mut() {
                if tab.follow_output {
                    tab.session.set_scrollback(0);
                } else {
                    tab.session.set_scrollback(tab.display_offset);
                }
                tab.display_offset = tab.session.scrollback();
                tab.follow_output = tab.display_offset == 0;
            }
            true
        } else {
            false
        }
    }

    fn close_active_terminal(&mut self) {
        let Some(active_id) = self.active_terminal_id.clone() else {
            return;
        };
        self.close_terminal_by_id(&active_id);
    }

    fn close_terminal_by_id(&mut self, tab_id: &str) {
        let was_active = self.active_terminal_id.as_deref() == Some(tab_id);
        let Some(index) = self
            .terminal_tabs
            .iter()
            .position(|tab| tab.id.as_str() == tab_id)
        else {
            return;
        };

        self.terminal_tabs.remove(index);

        if self.terminal_tabs.is_empty() {
            self.active_terminal_id = None;
            self.bottom_dock_visible = false;
            self.workspace.hide_panel(TERMINAL_PANEL_ID);
            return;
        }

        if was_active {
            let next_index = index.min(self.terminal_tabs.len().saturating_sub(1));
            self.active_terminal_id = self.terminal_tabs.get(next_index).map(|tab| tab.id.clone());
            self.bottom_dock_visible = true;
            self.workspace.focus_panel(TERMINAL_PANEL_ID);
        }
    }

    fn scroll_active_terminal(&mut self, lines: i32) {
        let Some(tab) = self.active_terminal_mut() else {
            return;
        };
        tab.display_offset = scroll_delta_to_display_offset(
            tab.display_offset,
            tab.session.max_safe_scrollback(),
            lines,
        );
        tab.session.set_scrollback(tab.display_offset);
        tab.display_offset = tab.session.scrollback();
        tab.follow_output = tab.display_offset == 0;
    }
}

#[cfg_attr(not(test), allow(dead_code))]
fn clamp_terminal_viewport_top(
    viewport_top: usize,
    total_lines: usize,
    viewport_height: usize,
) -> usize {
    viewport_top.min(live_bottom_viewport_top(total_lines, viewport_height))
}

#[cfg_attr(not(test), allow(dead_code))]
fn live_bottom_viewport_top(total_lines: usize, viewport_height: usize) -> usize {
    total_lines.saturating_sub(viewport_height)
}

#[cfg_attr(not(test), allow(dead_code))]
fn is_live_bottom_viewport_top(
    viewport_top: usize,
    total_lines: usize,
    viewport_height: usize,
) -> bool {
    clamp_terminal_viewport_top(viewport_top, total_lines, viewport_height)
        == live_bottom_viewport_top(total_lines, viewport_height)
}

#[cfg_attr(not(test), allow(dead_code))]
fn next_viewport_top_before_input(
    _current_top: usize,
    total_lines: usize,
    viewport_height: usize,
) -> usize {
    live_bottom_viewport_top(total_lines, viewport_height)
}

#[cfg_attr(not(test), allow(dead_code))]
fn scroll_delta_to_viewport_top(
    current_top: usize,
    total_lines: usize,
    viewport_height: usize,
    lines: i32,
) -> usize {
    let live_bottom_top = live_bottom_viewport_top(total_lines, viewport_height);
    let current_top = clamp_terminal_viewport_top(current_top, total_lines, viewport_height);

    if lines == i32::MIN {
        return live_bottom_top;
    }

    if lines >= 0 {
        current_top.saturating_sub(lines as usize)
    } else {
        current_top
            .saturating_add((-lines) as usize)
            .min(live_bottom_top)
    }
}

fn scroll_delta_to_display_offset(
    current_offset: usize,
    max_offset: usize,
    lines: i32,
) -> usize {
    if lines == i32::MIN {
        return 0;
    }

    if lines >= 0 {
        current_offset.saturating_add(lines as usize).min(max_offset)
    } else {
        current_offset.saturating_sub((-lines) as usize)
    }
}

fn clamp_px(value: gpui::Pixels, min: f32, max: f32) -> gpui::Pixels {
    px(f32::from(value).clamp(min, max))
}

struct WorkspaceMeta {
    name: String,
    root: String,
}

fn workspace_meta() -> WorkspaceMeta {
    let path = std::env::current_dir().ok();
    let name = path
        .as_deref()
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .unwrap_or(AppFrame::title())
        .to_owned();
    let root = path
        .as_deref()
        .and_then(Path::to_str)
        .unwrap_or("")
        .to_owned();
    WorkspaceMeta { name, root }
}

fn build_title_bar_tabs(
    center_panel: Option<&str>,
    _right_panel: Option<&str>,
    _bottom_panel: Option<&str>,
) -> Vec<TitleBarTab> {
    center_panel
        .filter(|label| !label.eq_ignore_ascii_case("terminal"))
        .map(|label| {
            vec![TitleBarTab {
                label: label.to_owned(),
                active: true,
            }]
        })
        .unwrap_or_default()
}

fn panel_title(
    workspace_state: &workspace::WorkspaceState,
    panel_id: Option<&str>,
) -> Option<String> {
    panel_id.and_then(|id| {
        workspace_state
            .registered_panels
            .get(id)
            .map(|descriptor| descriptor.title.clone())
    })
}

fn active_panel_title(
    workspace_state: &workspace::WorkspaceState,
    placement: DockPlacement,
) -> Option<String> {
    active_panel_id(workspace_state, placement)
        .and_then(|panel_key| panel_title(workspace_state, Some(panel_key.as_str())))
}

fn active_panel_id(
    workspace_state: &workspace::WorkspaceState,
    placement: DockPlacement,
) -> Option<String> {
    workspace_state
        .dock_layout
        .active_panel(placement)
        .map(|panel_key| panel_key.as_str().to_owned())
}

fn terminal_tab_title(tab: &TerminalTabRuntime) -> String {
    let snapshot = tab.session.snapshot();
    compact_terminal_title(
        snapshot.title.trim(),
        snapshot.shell_name.as_str(),
        snapshot.cwd.as_str(),
        tab.label.as_str(),
    )
}

fn compact_terminal_title(
    title: &str,
    shell_name: &str,
    cwd: &str,
    fallback_label: &str,
) -> String {
    let normalized_title = title.trim();
    if normalized_title.is_empty() || normalized_title.eq_ignore_ascii_case("terminal") {
        return fallback_label.to_owned();
    }

    if let Some(shell_title) = compact_shell_path_title(normalized_title) {
        return shell_title;
    }

    if let Some(directory_name) = compact_directory_title(normalized_title) {
        return directory_name;
    }

    if normalized_title.len() > 36 {
        if let Some(directory_name) = compact_directory_title(cwd) {
            return format!("{directory_name} ({})", compact_shell_name(shell_name));
        }
    }

    normalized_title.to_owned()
}

fn compact_shell_path_title(title: &str) -> Option<String> {
    let path = Path::new(title);
    let extension = path.extension()?.to_str()?;
    if !extension.eq_ignore_ascii_case("exe") {
        return None;
    }

    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map(|stem| stem.to_owned())
}

fn compact_directory_title(path_like: &str) -> Option<String> {
    if !path_like.contains(['\\', '/']) {
        return None;
    }

    let path = Path::new(path_like.trim_end_matches(['\\', '/']));
    path.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .map(|name| name.to_owned())
}

fn compact_shell_name(shell_name: &str) -> String {
    Path::new(shell_name)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .unwrap_or(shell_name)
        .to_owned()
}

fn shell_terminal_cell(cell: TerminalCell) -> ui::ShellTerminalCell {
    ui::ShellTerminalCell {
        text: cell.text,
        foreground: cell.foreground,
        background: cell.background,
        bold: cell.bold,
        underline: cell.underline,
    }
}

fn pixels_to_terminal_lines(delta: gpui::Pixels) -> i32 {
    let pixels = f32::from(delta);
    if pixels.abs() < 1.0 {
        0
    } else {
        (pixels / TERMINAL_LINE_HEIGHT).round() as i32
    }
}

fn terminal_rows_for_bottom_dock_height(bottom_dock_height: gpui::Pixels) -> usize {
    let viewport_height = (f32::from(bottom_dock_height)
        - TERMINAL_HEADER_HEIGHT
        - TERMINAL_VERTICAL_PADDING)
        .max(TERMINAL_LINE_HEIGHT * MIN_TERMINAL_ROWS as f32);

    (viewport_height / TERMINAL_LINE_HEIGHT).floor() as usize
}

fn terminal_viewport_lines_for_key_event(event: &KeyDownEvent, visible_rows: usize) -> Option<i32> {
    if !event.keystroke.modifiers.shift {
        return None;
    }

    match event.keystroke.key.as_str() {
        "pageup" => Some(visible_rows.max(1) as i32),
        "pagedown" => Some(-((visible_rows.max(1)) as i32)),
        _ => None,
    }
}

fn key_event_to_bytes_with_mode(event: &KeyDownEvent, application_cursor: bool) -> Option<Vec<u8>> {
    let keystroke = &event.keystroke;
    let key = keystroke.key.as_str();

    let bytes = match key {
        "enter" => b"\r".to_vec(),
        "backspace" => vec![0x08],
        "tab" => b"\t".to_vec(),
        "delete" => b"\x1b[3~".to_vec(),
        "escape" => vec![0x1b],
        "space" => b" ".to_vec(),
        "up" => {
            if application_cursor {
                b"\x1bOA".to_vec()
            } else {
                b"\x1b[A".to_vec()
            }
        }
        "down" => {
            if application_cursor {
                b"\x1bOB".to_vec()
            } else {
                b"\x1b[B".to_vec()
            }
        }
        "right" => {
            if application_cursor {
                b"\x1bOC".to_vec()
            } else {
                b"\x1b[C".to_vec()
            }
        }
        "left" => {
            if application_cursor {
                b"\x1bOD".to_vec()
            } else {
                b"\x1b[D".to_vec()
            }
        }
        "home" => b"\x1b[H".to_vec(),
        "end" => b"\x1b[F".to_vec(),
        "pageup" => b"\x1b[5~".to_vec(),
        "pagedown" => b"\x1b[6~".to_vec(),
        _ if keystroke.modifiers.control && key.len() == 1 => {
            let character = key.as_bytes()[0].to_ascii_lowercase();
            vec![character.saturating_sub(b'a') + 1]
        }
        _ => keystroke.key_char.as_ref()?.as_bytes().to_vec(),
    };

    if keystroke.modifiers.alt {
        let mut prefixed = vec![0x1b];
        prefixed.extend(bytes);
        Some(prefixed)
    } else {
        Some(bytes)
    }
}

fn shell_theme_from_definition(theme: &ThemeDefinition) -> ShellTheme {
    ShellTheme {
        app_background: theme.token_hex("app.background", 0x0f1419),
        title_bar_background: theme.token_hex("title_bar.background", 0x10161d),
        title_bar_foreground: theme.token_hex("title_bar.foreground", 0xe1e8ef),
        title_bar_border: theme.token_hex("title_bar.border", 0x1f2a33),
        window_control_foreground: theme.token_hex("window_control.foreground", 0xf2f7fb),
        window_control_hover: theme.token_hex("window_control.hover", 0x2b3844),
        window_control_close_hover: theme.token_hex("window_control.close_hover", 0xc42b1c),
        sidebar_background: theme.token_hex("sidebar.background", 0x121921),
        sidebar_foreground: theme.token_hex("sidebar.foreground", 0xcdd8e2),
        sidebar_muted: theme.token_hex("sidebar.muted", 0x8092a2),
        dock_background: theme.token_hex("dock.background", 0x0f1419),
        dock_header: theme.token_hex("dock.header", 0x10161d),
        border: theme.token_hex("border", 0x1c2730),
        resize_handle_hover: theme.token_hex("resize_handle.hover", 0x4d8fd6),
        status_bar_background: theme.token_hex("status_bar.background", 0x111820),
        status_bar_foreground: theme.token_hex("status_bar.foreground", 0x8fa2b4),
        status_bar_active: theme.token_hex("status_bar.active", 0xdce7f0),
        status_bar_button: theme.token_hex("status_bar.button", 0x141d26),
        status_bar_button_active: theme.token_hex("status_bar.button_active", 0x1b2834),
        status_bar_button_hover: theme.token_hex("status_bar.button_hover", 0x24313d),
    }
}

fn render_platform_title_bar(
    _title: &'static str,
    workspace_meta: &WorkspaceMeta,
    tabs: &[TitleBarTab],
    window: &Window,
    theme: ShellTheme,
) -> impl IntoElement {
    div()
        .h(px(36.))
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .bg(rgb(theme.title_bar_background))
        .border_b_1()
        .border_color(rgb(theme.title_bar_border))
        .child(
            div()
                .h_full()
                .flex()
                .items_center()
                .flex_1()
                .gap_0p5()
                .px_1p5()
                .text_sm()
                .text_color(rgb(theme.title_bar_foreground))
                .window_control_area(WindowControlArea::Drag)
                .child(titlebar_icon_button(
                    "title-bar-menu",
                    IconName::Menu,
                    false,
                    theme,
                ))
                .child(title_bar_workspace_identity(workspace_meta, theme))
                .child(title_bar_divider(theme))
                .child(
                    div()
                        .flex()
                        .items_end()
                        .gap_0p5()
                        .pt_1()
                        .children(
                            tabs.iter().map(|tab| {
                                titlebar_tab_chip(&tab.label, tab.active, theme).into_any_element()
                            }),
                        ),
                ),
        )
        .child(
            div()
                .h_full()
                .flex()
                .items_center()
                .flex_none()
                .bg(rgb(theme.title_bar_background))
                .child(render_windows_window_controls(window, theme)),
        )
}

fn title_bar_workspace_identity(workspace_meta: &WorkspaceMeta, theme: ShellTheme) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap_1p5()
        .child(titlebar_label(&workspace_meta.name, theme))
}

fn title_bar_divider(theme: ShellTheme) -> impl IntoElement {
    div()
        .h(px(16.))
        .w(px(1.))
        .bg(rgb(theme.title_bar_border))
}

fn render_windows_window_controls(window: &Window, theme: ShellTheme) -> impl IntoElement {
    div()
        .id("windows-window-controls")
        .font_family("Segoe Fluent Icons")
        .flex()
        .flex_row()
        .justify_center()
        .content_stretch()
        .max_h(px(36.))
        .min_h(px(36.))
        .child(windows_caption_button(
            "minimize",
            "\u{e921}",
            WindowControlArea::Min,
            false,
            theme,
        ))
        .child(windows_caption_button(
            if window.is_maximized() {
                "restore"
            } else {
                "maximize"
            },
            if window.is_maximized() {
                "\u{e923}"
            } else {
                "\u{e922}"
            },
            WindowControlArea::Max,
            false,
            theme,
        ))
        .child(windows_caption_button(
            "close",
            "\u{e8bb}",
            WindowControlArea::Close,
            true,
            theme,
        ))
}

fn windows_caption_button(
    id: &'static str,
    glyph: &'static str,
    control_area: WindowControlArea,
    is_close: bool,
    theme: ShellTheme,
) -> impl IntoElement {
    let background = theme.title_bar_background;
    let foreground = theme.window_control_foreground;
    let hover_background = if is_close {
        theme.window_control_close_hover
    } else {
        theme.window_control_hover
    };
    div()
        .id(id)
        .h_full()
        .w(px(44.))
        .flex()
        .items_center()
        .justify_center()
        .bg(rgb(background))
        .text_color(rgb(foreground))
        .text_size(px(11.))
        .hover(move |style| {
            style
                .bg(rgb(hover_background))
                .text_color(rgb(0xffffff))
        })
        .window_control_area(control_area)
        .child(glyph)
}

#[cfg(test)]
mod tests {
    use super::{
        clamp_terminal_viewport_top, compact_terminal_title, is_live_bottom_viewport_top,
        next_viewport_top_before_input, scroll_delta_to_display_offset,
        scroll_delta_to_viewport_top,
    };
    use gpui::px;

    #[test]
    fn page_navigation_uses_total_history_bounds() {
        assert_eq!(clamp_terminal_viewport_top(0, 120, 20), 0);
        assert_eq!(clamp_terminal_viewport_top(50, 120, 20), 50);
        assert_eq!(clamp_terminal_viewport_top(200, 120, 20), 100);
    }

    #[test]
    fn typing_while_scrolled_up_resets_terminal_to_bottom() {
        let next_top = next_viewport_top_before_input(35, 120, 20);
        assert_eq!(next_top, 100);
    }

    #[test]
    fn scroll_direction_keeps_zero_at_live_bottom() {
        assert_eq!(scroll_delta_to_viewport_top(100, 120, 20, 8), 92);
        assert_eq!(scroll_delta_to_viewport_top(92, 120, 20, -8), 100);
    }

    #[test]
    fn live_bottom_detection_matches_terminal_history_bounds() {
        assert!(is_live_bottom_viewport_top(100, 120, 20));
        assert!(!is_live_bottom_viewport_top(99, 120, 20));
        assert!(is_live_bottom_viewport_top(0, 10, 20));
    }

    #[test]
    fn scroll_direction_keeps_zero_at_live_bottom_for_display_offset() {
        assert_eq!(scroll_delta_to_display_offset(0, 20, 8), 8);
        assert_eq!(scroll_delta_to_display_offset(8, 20, -8), 0);
        assert_eq!(scroll_delta_to_display_offset(8, 20, i32::MIN), 0);
    }

    #[test]
    fn terminal_rows_subtract_header_and_padding() {
        assert_eq!(super::terminal_rows_for_bottom_dock_height(px(200.)), 8);
        assert_eq!(super::terminal_rows_for_bottom_dock_height(px(260.)), 11);
    }

    #[test]
    fn terminal_titles_collapse_shell_paths_to_shell_name() {
        assert_eq!(
            compact_terminal_title(
                "C:\\Program Files\\PowerShell\\7\\pwsh.exe",
                "C:\\Program Files\\PowerShell\\7\\pwsh.exe",
                "E:\\code\\workspace",
                "Terminal 1",
            ),
            "pwsh"
        );
    }

    #[test]
    fn terminal_titles_fall_back_to_workspace_name_when_raw_title_is_too_long() {
        assert_eq!(
            compact_terminal_title(
                "this is a very long terminal title that should not be shown verbatim",
                "C:\\Program Files\\PowerShell\\7\\pwsh.exe",
                "E:\\code\\workspace",
                "Terminal 1",
            ),
            "workspace (pwsh)"
        );
    }

    #[test]
    fn terminal_titles_keep_short_custom_titles() {
        assert_eq!(
            compact_terminal_title(
                "server logs",
                "pwsh.exe",
                "E:\\code\\workspace",
                "Terminal 1"
            ),
            "server logs"
        );
    }
}
