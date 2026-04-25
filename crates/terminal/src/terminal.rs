use alacritty_terminal::{
    event::{Event as AlacrittyEvent, EventListener},
    grid::{Dimensions, Indexed, Scroll},
    term::{
        self, Config, Term, TermMode,
        cell::{Cell as AlacrittyCell, Flags},
        color::Colors,
    },
    vte::ansi::{Color, CursorShape, NamedColor, Processor, Rgb, StdSyncHandler},
};
use panel::{DockPlacement, PanelDescriptor, PanelTypeId};
use portable_pty::{CommandBuilder, MasterPty, PtySize, native_pty_system};
use std::cmp::Ordering;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;

pub const TERMINAL_PANEL_ID: &str = "terminal.panel";
const DEFAULT_SCROLLBACK_LINES: usize = 16_384;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalSnapshot {
    pub shell_name: String,
    pub cwd: String,
    pub status: String,
    pub title: String,
    pub rows: u16,
    pub cols: u16,
    pub cursor_row: u16,
    pub cursor_col: u16,
    pub cursor_hidden: bool,
    pub application_cursor: bool,
    pub total_lines: usize,
    pub viewport_top: usize,
    pub viewport_height: usize,
    pub can_scroll_up: bool,
    pub can_scroll_down: bool,
    pub scrollback: usize,
    pub visible_cells: Vec<Vec<TerminalCell>>,
    pub visible_lines: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalCell {
    pub text: String,
    pub foreground: Option<u32>,
    pub background: Option<u32>,
    pub bold: bool,
    pub underline: bool,
}

pub fn terminal_panel_descriptor() -> PanelDescriptor {
    PanelDescriptor::singleton(
        PanelTypeId::from_static(TERMINAL_PANEL_ID),
        "Terminal",
        DockPlacement::Bottom,
    )
}

pub struct TerminalSession {
    shell_name: String,
    cwd: String,
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    child: Box<dyn portable_pty::Child + Send + Sync>,
    terminal: TerminalBuffer,
    output_rx: Receiver<Vec<u8>>,
    status: String,
}

struct TerminalBuffer {
    term: Term<SessionEventListener>,
    event_state: SessionEventState,
}

#[derive(Clone, Default)]
struct SessionEventState {
    title: Arc<Mutex<Option<String>>>,
}

#[derive(Clone, Default)]
struct SessionEventListener {
    state: SessionEventState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TerminalDimensions {
    rows: u16,
    cols: u16,
}

impl TerminalDimensions {
    const fn new(rows: u16, cols: u16) -> Self {
        Self { rows, cols }
    }
}

impl Dimensions for TerminalDimensions {
    fn total_lines(&self) -> usize {
        self.screen_lines()
    }

    fn screen_lines(&self) -> usize {
        usize::from(self.rows)
    }

    fn columns(&self) -> usize {
        usize::from(self.cols)
    }
}

impl EventListener for SessionEventListener {
    fn send_event(&self, event: AlacrittyEvent) {
        match event {
            AlacrittyEvent::Title(title) => {
                *lock_or_recover(&self.state.title) = Some(title);
            }
            AlacrittyEvent::ResetTitle => {
                *lock_or_recover(&self.state.title) = None;
            }
            _ => {}
        }
    }
}

impl TerminalBuffer {
    fn new(rows: u16, cols: u16, scrollback_lines: usize) -> Self {
        let event_state = SessionEventState::default();
        let listener = SessionEventListener {
            state: event_state.clone(),
        };
        let term = Term::new(
            Config {
                scrolling_history: scrollback_lines,
                ..Config::default()
            },
            &TerminalDimensions::new(rows, cols),
            listener,
        );

        Self { term, event_state }
    }

    fn process_output(&mut self, bytes: &[u8]) {
        let mut processor = Processor::<StdSyncHandler>::new();
        processor.advance(&mut self.term, bytes);
    }

    fn resize(&mut self, rows: u16, cols: u16) {
        self.term.resize(TerminalDimensions::new(rows, cols));
    }

    fn scroll_to_bottom(&mut self) {
        self.term.scroll_display(Scroll::Bottom);
    }

    fn set_scrollback(&mut self, rows: usize) {
        let requested_offset = rows.min(self.max_scrollback());
        let current_offset = self.scrollback();

        match requested_offset.cmp(&current_offset) {
            Ordering::Greater => {
                let delta = requested_offset.saturating_sub(current_offset);
                self.term.scroll_display(Scroll::Delta(delta as i32));
            }
            Ordering::Less => {
                let delta = current_offset.saturating_sub(requested_offset);
                self.term.scroll_display(Scroll::Delta(-(delta as i32)));
            }
            Ordering::Equal => {}
        }
    }

    fn scrollback(&self) -> usize {
        self.term.grid().display_offset()
    }

    fn max_scrollback(&self) -> usize {
        self.term.total_lines().saturating_sub(self.term.screen_lines())
    }

    fn total_lines(&self) -> usize {
        self.term.total_lines()
    }

    fn viewport_height(&self) -> usize {
        self.term.screen_lines()
    }

    fn viewport_top(&self) -> usize {
        self.total_lines()
            .saturating_sub(self.viewport_height())
            .saturating_sub(self.scrollback())
    }

    fn application_cursor(&self) -> bool {
        self.term.mode().contains(TermMode::APP_CURSOR)
    }

    fn title(&self) -> String {
        lock_or_recover(&self.event_state.title)
            .clone()
            .filter(|title| !title.trim().is_empty())
            .unwrap_or_else(|| "Terminal".to_owned())
    }

    fn snapshot(
        &self,
        shell_name: String,
        cwd: String,
        status: String,
    ) -> TerminalSnapshot {
        let rows = self.term.screen_lines();
        let cols = self.term.columns();
        let total_lines = self.total_lines();
        let viewport_height = self.viewport_height();
        let scrollback = self.scrollback();
        let viewport_top = self.viewport_top();
        let content = self.term.renderable_content();
        let cursor_row = u16::try_from(content.cursor.point.line.0.max(0)).unwrap_or(0);
        let cursor_col = u16::try_from(content.cursor.point.column.0).unwrap_or(0);
        let cursor_hidden = content.cursor.shape == CursorShape::Hidden;
        let application_cursor = content.mode.contains(TermMode::APP_CURSOR);
        let (visible_cells, visible_lines) = render_visible_content(
            content.display_iter,
            content.display_offset,
            content.colors,
            rows,
            cols,
        );

        TerminalSnapshot {
            shell_name,
            cwd,
            status,
            title: self.title(),
            rows: u16::try_from(rows).unwrap_or(u16::MAX),
            cols: u16::try_from(cols).unwrap_or(u16::MAX),
            cursor_row,
            cursor_col,
            cursor_hidden,
            application_cursor,
            total_lines,
            viewport_top,
            viewport_height,
            can_scroll_up: viewport_top > 0,
            can_scroll_down: scrollback > 0,
            scrollback,
            visible_cells,
            visible_lines,
        }
    }
}

impl TerminalSession {
    pub fn spawn_local() -> Result<Self, String> {
        let pty_system = native_pty_system();
        let cwd = std::env::current_dir().ok();
        let shell_name = local_shell_candidates()
            .first()
            .copied()
            .unwrap_or(default_shell_name());

        let pair = pty_system
            .openpty(PtySize {
                rows: 40,
                cols: 120,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| format!("failed to create PTY: {error}"))?;

        let mut builder = CommandBuilder::new(shell_name);
        if let Some(cwd) = cwd.as_ref() {
            builder.cwd(cwd);
        }
        configure_command_builder(&mut builder, shell_name);

        let child = pair
            .slave
            .spawn_command(builder)
            .map_err(|error| format!("failed to spawn shell: {error}"))?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|error| format!("failed to open PTY writer: {error}"))?;
        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|error| format!("failed to open PTY reader: {error}"))?;

        let (output_tx, output_rx) = channel();
        spawn_output_reader(reader, output_tx);

        Ok(Self {
            shell_name: shell_name.to_owned(),
            cwd: cwd
                .as_deref()
                .and_then(Path::to_str)
                .unwrap_or("")
                .to_owned(),
            master: pair.master,
            writer,
            child,
            terminal: TerminalBuffer::new(40, 120, DEFAULT_SCROLLBACK_LINES),
            output_rx,
            status: "running".to_owned(),
        })
    }

    pub fn refresh(&mut self) {
        while let Ok(chunk) = self.output_rx.try_recv() {
            self.terminal.process_output(&chunk);
        }

        match self.child.try_wait() {
            Ok(Some(status)) => {
                self.status = format!("exited {status}");
            }
            Ok(None) => {
                self.status = "running".to_owned();
            }
            Err(error) => {
                self.status = format!("status unavailable: {error}");
            }
        }
    }

    pub fn is_running(&mut self) -> bool {
        self.refresh();
        self.status == "running"
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        self.terminal.scroll_to_bottom();
        self.writer
            .write_all(bytes)
            .map_err(|error| format!("write failed: {error}"))?;
        self.writer
            .flush()
            .map_err(|error| format!("flush failed: {error}"))?;
        Ok(())
    }

    pub fn application_cursor(&self) -> bool {
        self.terminal.application_cursor()
    }

    pub fn resize(&mut self, rows: u16, cols: u16) -> Result<(), String> {
        self.terminal.resize(rows, cols);
        self.master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| format!("resize failed: {error}"))
    }

    pub fn set_scrollback(&mut self, rows: usize) {
        self.terminal.set_scrollback(rows);
    }

    pub fn scrollback(&self) -> usize {
        self.terminal.scrollback()
    }

    pub fn max_safe_scrollback(&self) -> usize {
        self.terminal.max_scrollback()
    }

    pub fn total_lines(&self) -> usize {
        self.terminal.total_lines()
    }

    pub fn viewport_top(&self) -> usize {
        self.terminal.viewport_top()
    }

    pub fn viewport_height(&self) -> usize {
        self.terminal.viewport_height()
    }

    pub fn max_viewport_top(&self) -> usize {
        self.total_lines().saturating_sub(self.viewport_height())
    }

    pub fn set_viewport_top(&mut self, viewport_top: usize) {
        let display_offset = self
            .max_viewport_top()
            .saturating_sub(viewport_top.min(self.max_viewport_top()));
        self.set_scrollback(display_offset);
    }

    pub fn snapshot(&self) -> TerminalSnapshot {
        self.terminal.snapshot(
            self.shell_name.clone(),
            self.cwd.clone(),
            self.status.clone(),
        )
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _result = self.child.kill();
    }
}

fn lock_or_recover<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn spawn_output_reader(reader: Box<dyn Read + Send>, sender: Sender<Vec<u8>>) {
    thread::spawn(move || {
        let mut reader = reader;
        let mut buffer = [0_u8; 4096];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(bytes_read) => {
                    if sender.send(buffer[..bytes_read].to_vec()).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });
}

fn render_visible_content(
    display_iter: alacritty_terminal::grid::GridIterator<'_, AlacrittyCell>,
    display_offset: usize,
    colors: &Colors,
    rows: usize,
    cols: usize,
) -> (Vec<Vec<TerminalCell>>, Vec<String>) {
    let mut visible_cells = vec![vec![blank_cell(None, None); cols]; rows];

    for Indexed { point, cell } in display_iter {
        let Some(viewport_point) = term::point_to_viewport(display_offset, point) else {
            continue;
        };

        if viewport_point.line >= rows || viewport_point.column.0 >= cols {
            continue;
        }

        if cell.flags.intersects(Flags::WIDE_CHAR_SPACER | Flags::LEADING_WIDE_CHAR_SPACER) {
            continue;
        }

        visible_cells[viewport_point.line][viewport_point.column.0] = render_cell(cell, colors);
    }

    let visible_lines = visible_cells
        .iter()
        .map(|row| {
            let mut rendered = String::new();
            for cell in row {
                rendered.push_str(&cell.text);
            }
            rendered
        })
        .collect();

    (visible_cells, visible_lines)
}

fn render_cell(cell: &AlacrittyCell, colors: &Colors) -> TerminalCell {
    let mut foreground = color_to_hex(cell.fg, colors);
    let mut background = color_to_hex(cell.bg, colors);
    if cell.flags.contains(Flags::INVERSE) {
        std::mem::swap(&mut foreground, &mut background);
    }

    TerminalCell {
        text: visible_cell_text(cell),
        foreground,
        background,
        bold: cell.flags.intersects(Flags::BOLD | Flags::DIM_BOLD),
        underline: cell.flags.intersects(Flags::ALL_UNDERLINES),
    }
}

fn visible_cell_text(cell: &AlacrittyCell) -> String {
    if cell.flags.contains(Flags::HIDDEN) {
        return " ".to_owned();
    }

    let mut text = String::new();
    text.push(cell.c);
    if let Some(zerowidth) = cell.zerowidth() {
        for character in zerowidth {
            text.push(*character);
        }
    }
    text
}

fn blank_cell(foreground: Option<u32>, background: Option<u32>) -> TerminalCell {
    TerminalCell {
        text: " ".to_owned(),
        foreground,
        background,
        bold: false,
        underline: false,
    }
}

fn color_to_hex(color: Color, colors: &Colors) -> Option<u32> {
    match color {
        Color::Named(named) => colors[named]
            .map(rgb_to_hex)
            .or_else(|| named_color_to_hex(named)),
        Color::Spec(rgb) => Some(rgb_to_hex(rgb)),
        Color::Indexed(index) => colors[index as usize]
            .map(rgb_to_hex)
            .or_else(|| Some(ansi_index_to_hex(index))),
    }
}

fn rgb_to_hex(rgb: Rgb) -> u32 {
    (u32::from(rgb.r) << 16) | (u32::from(rgb.g) << 8) | u32::from(rgb.b)
}

fn named_color_to_hex(color: NamedColor) -> Option<u32> {
    match color {
        NamedColor::Black => Some(ansi_index_to_hex(0)),
        NamedColor::Red => Some(ansi_index_to_hex(1)),
        NamedColor::Green => Some(ansi_index_to_hex(2)),
        NamedColor::Yellow => Some(ansi_index_to_hex(3)),
        NamedColor::Blue => Some(ansi_index_to_hex(4)),
        NamedColor::Magenta => Some(ansi_index_to_hex(5)),
        NamedColor::Cyan => Some(ansi_index_to_hex(6)),
        NamedColor::White => Some(ansi_index_to_hex(7)),
        NamedColor::BrightBlack => Some(ansi_index_to_hex(8)),
        NamedColor::BrightRed => Some(ansi_index_to_hex(9)),
        NamedColor::BrightGreen => Some(ansi_index_to_hex(10)),
        NamedColor::BrightYellow => Some(ansi_index_to_hex(11)),
        NamedColor::BrightBlue => Some(ansi_index_to_hex(12)),
        NamedColor::BrightMagenta => Some(ansi_index_to_hex(13)),
        NamedColor::BrightCyan => Some(ansi_index_to_hex(14)),
        NamedColor::BrightWhite => Some(ansi_index_to_hex(15)),
        NamedColor::DimBlack => Some(ansi_index_to_hex(0)),
        NamedColor::DimRed => Some(ansi_index_to_hex(1)),
        NamedColor::DimGreen => Some(ansi_index_to_hex(2)),
        NamedColor::DimYellow => Some(ansi_index_to_hex(3)),
        NamedColor::DimBlue => Some(ansi_index_to_hex(4)),
        NamedColor::DimMagenta => Some(ansi_index_to_hex(5)),
        NamedColor::DimCyan => Some(ansi_index_to_hex(6)),
        NamedColor::DimWhite => Some(ansi_index_to_hex(7)),
        NamedColor::Foreground
        | NamedColor::Background
        | NamedColor::Cursor
        | NamedColor::BrightForeground
        | NamedColor::DimForeground => None,
    }
}

fn ansi_index_to_hex(index: u8) -> u32 {
    match index {
        0 => 0x000000,
        1 => 0xcd3131,
        2 => 0x0dbc79,
        3 => 0xe5e510,
        4 => 0x2472c8,
        5 => 0xbc3fbc,
        6 => 0x11a8cd,
        7 => 0xe5e5e5,
        8 => 0x666666,
        9 => 0xf14c4c,
        10 => 0x23d18b,
        11 => 0xf5f543,
        12 => 0x3b8eea,
        13 => 0xd670d6,
        14 => 0x29b8db,
        15 => 0xe5e5e5,
        16..=231 => ansi_cube_color(index),
        232..=255 => ansi_grayscale_color(index),
    }
}

fn ansi_cube_color(index: u8) -> u32 {
    let value = index - 16;
    let red = value / 36;
    let green = (value % 36) / 6;
    let blue = value % 6;

    (u32::from(cube_component(red)) << 16)
        | (u32::from(cube_component(green)) << 8)
        | u32::from(cube_component(blue))
}

fn cube_component(component: u8) -> u8 {
    if component == 0 {
        0
    } else {
        55 + component * 40
    }
}

fn ansi_grayscale_color(index: u8) -> u32 {
    let value = 8 + (index - 232) * 10;
    (u32::from(value) << 16) | (u32::from(value) << 8) | u32::from(value)
}

#[cfg(windows)]
fn local_shell_candidates() -> &'static [&'static str] {
    &["pwsh.exe", "powershell.exe", "cmd.exe"]
}

#[cfg(not(windows))]
fn local_shell_candidates() -> &'static [&'static str] {
    &["zsh", "bash", "sh"]
}

#[cfg(windows)]
fn default_shell_name() -> &'static str {
    "pwsh.exe"
}

#[cfg(not(windows))]
fn default_shell_name() -> &'static str {
    "bash"
}

#[cfg(windows)]
fn configure_command_builder(builder: &mut CommandBuilder, shell: &str) {
    match shell {
        "pwsh.exe" | "powershell.exe" => {
            builder.arg("-NoLogo");
            builder.arg("-NoExit");
        }
        _ => {}
    }
}

#[cfg(not(windows))]
fn configure_command_builder(_builder: &mut CommandBuilder, _shell: &str) {}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_SCROLLBACK_LINES, TERMINAL_PANEL_ID, TerminalBuffer, ansi_index_to_hex,
        color_to_hex, terminal_panel_descriptor,
    };
    use alacritty_terminal::{
        term::color::Colors,
        vte::ansi::{Color, NamedColor, Rgb},
    };
    use panel::DockPlacement;

    #[test]
    fn terminal_panel_defaults_to_bottom_singleton() {
        let descriptor = terminal_panel_descriptor();
        assert_eq!(descriptor.id.as_str(), TERMINAL_PANEL_ID);
        assert_eq!(descriptor.dock, DockPlacement::Bottom);
    }

    #[test]
    fn maps_terminal_palette_indexes_to_rgb_hex() {
        let colors = Colors::default();

        assert_eq!(color_to_hex(Color::Indexed(1), &colors), Some(0xcd3131));
        assert_eq!(color_to_hex(Color::Named(NamedColor::Foreground), &colors), None);
        assert_eq!(
            color_to_hex(Color::Spec(Rgb { r: 0x12, g: 0x34, b: 0x56 }), &colors),
            Some(0x123456)
        );
        assert_eq!(ansi_index_to_hex(16), 0x000000);
        assert_eq!(ansi_index_to_hex(231), 0xffffff);
        assert_eq!(ansi_index_to_hex(244), 0x808080);
    }

    #[test]
    fn scrollback_reaches_full_history_not_viewport_height() {
        let mut terminal = TerminalBuffer::new(3, 20, DEFAULT_SCROLLBACK_LINES);
        terminal.process_output(b"one\r\ntwo\r\nthree\r\nfour\r\nfive\r\nsix\r\nseven\r\neight");

        assert_eq!(terminal.total_lines(), 8);
        assert_eq!(terminal.viewport_height(), 3);
        assert_eq!(terminal.max_scrollback(), 5);

        terminal.set_scrollback(usize::MAX);

        assert_eq!(terminal.scrollback(), 5);
        assert_eq!(terminal.viewport_top(), 0);
    }

    #[test]
    fn resize_uses_total_history_for_scroll_range() {
        let mut terminal = TerminalBuffer::new(3, 20, DEFAULT_SCROLLBACK_LINES);
        terminal.process_output(b"one\r\ntwo\r\nthree\r\nfour\r\nfive\r\nsix\r\nseven\r\neight");

        assert_eq!(terminal.max_scrollback(), 5);

        terminal.resize(5, 20);

        assert_eq!(terminal.total_lines(), 8);
        assert_eq!(terminal.viewport_height(), 5);
        assert_eq!(terminal.max_scrollback(), 3);
    }

    #[test]
    fn scrolling_back_to_bottom_clears_display_offset() {
        let mut terminal = TerminalBuffer::new(3, 20, DEFAULT_SCROLLBACK_LINES);
        terminal.process_output(b"one\r\ntwo\r\nthree\r\nfour\r\nfive\r\nsix");
        terminal.set_scrollback(usize::MAX);

        assert!(terminal.scrollback() > 0);

        terminal.scroll_to_bottom();

        assert_eq!(terminal.scrollback(), 0);
        assert_eq!(terminal.viewport_top(), 3);
    }
}
