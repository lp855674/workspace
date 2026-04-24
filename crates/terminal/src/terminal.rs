use panel::{DockPlacement, PanelDescriptor, PanelTypeId};
use portable_pty::{CommandBuilder, MasterPty, PtySize, native_pty_system};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use vt100::Color;
use vt100::Parser;
use vt100::Screen;

pub const TERMINAL_PANEL_ID: &str = "terminal.panel";

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
    parser: Parser,
    viewport: TerminalViewport,
    output_rx: Receiver<Vec<u8>>,
    status: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TerminalViewport {
    total_lines: usize,
    viewport_top: usize,
    viewport_height: usize,
}

impl TerminalViewport {
    fn viewport_height(self) -> usize {
        self.viewport_height
    }

    fn max_viewport_top(self) -> usize {
        self.total_lines.saturating_sub(self.viewport_height)
    }

    fn set_viewport_top(self, viewport_top: usize) -> Self {
        Self {
            viewport_top: clamp_viewport_top(
                viewport_top,
                self.total_lines,
                self.viewport_height,
            ),
            ..self
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    fn with_height(self, viewport_height: usize) -> Self {
        Self {
            viewport_height,
            ..self
        }
        .set_viewport_top(self.viewport_top)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    fn jump_to_bottom(self) -> Self {
        self.set_viewport_top(self.max_viewport_top())
    }

    fn is_away_from_bottom(self) -> bool {
        self.viewport_top < self.max_viewport_top()
    }

    fn scrollback_offset(self) -> usize {
        self.max_viewport_top().saturating_sub(self.viewport_top)
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
            parser: Parser::new(40, 120, 16_384),
            viewport: TerminalViewport {
                total_lines: 40,
                viewport_top: 0,
                viewport_height: 40,
            },
            output_rx,
            status: "running".to_owned(),
        })
    }

    pub fn refresh(&mut self) {
        while let Ok(chunk) = self.output_rx.try_recv() {
            self.parser.process(&chunk);
        }
        self.sync_viewport_from_parser();

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
        self.writer
            .write_all(bytes)
            .map_err(|error| format!("write failed: {error}"))?;
        self.writer
            .flush()
            .map_err(|error| format!("flush failed: {error}"))?;
        Ok(())
    }

    pub fn application_cursor(&self) -> bool {
        self.parser.screen().application_cursor()
    }

    pub fn resize(&mut self, rows: u16, cols: u16) -> Result<(), String> {
        self.parser.set_size(rows, cols);
        let result = self
            .master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| format!("resize failed: {error}"));
        self.sync_viewport_from_parser();
        result
    }

    pub fn set_scrollback(&mut self, rows: usize) {
        self.sync_viewport_from_parser();
        let viewport_top = self.max_viewport_top().saturating_sub(
            clamp_scrollback_offset(rows, self.max_safe_scrollback()),
        );
        self.set_viewport_top(viewport_top);
    }

    pub fn scrollback(&self) -> usize {
        self.viewport.scrollback_offset()
    }

    pub fn max_safe_scrollback(&self) -> usize {
        self.max_viewport_top()
    }

    pub fn total_lines(&self) -> usize {
        self.viewport.total_lines
    }

    pub fn viewport_height(&self) -> usize {
        self.viewport.viewport_height()
    }

    pub fn max_viewport_top(&self) -> usize {
        self.viewport.max_viewport_top()
    }

    pub fn set_viewport_top(&mut self, viewport_top: usize) {
        self.viewport = self.viewport.set_viewport_top(viewport_top);
        self.parser
            .set_scrollback(self.viewport.scrollback_offset());
    }

    pub fn snapshot(&self) -> TerminalSnapshot {
        let screen = self.parser.screen();
        let (rows, cols) = screen.size();
        let (cursor_row, cursor_col) = screen.cursor_position();
        let viewport = self.viewport;
        TerminalSnapshot {
            shell_name: self.shell_name.clone(),
            cwd: self.cwd.clone(),
            status: self.status.clone(),
            title: if screen.title().is_empty() {
                "Terminal".to_owned()
            } else {
                screen.title().to_owned()
            },
            rows,
            cols,
            cursor_row,
            cursor_col,
            cursor_hidden: screen.hide_cursor(),
            application_cursor: screen.application_cursor(),
            total_lines: viewport.total_lines,
            viewport_top: viewport.viewport_top,
            viewport_height: viewport.viewport_height(),
            can_scroll_up: viewport.viewport_top > 0,
            can_scroll_down: viewport.is_away_from_bottom(),
            scrollback: viewport.scrollback_offset(),
            visible_cells: render_visible_cells(screen, rows, cols),
            visible_lines: render_visible_lines(screen, rows, cols),
        }
    }

    fn sync_viewport_from_parser(&mut self) {
        self.viewport = snapshot_viewport_from_parser(&mut self.parser);
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _result = self.child.kill();
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

fn render_visible_lines(screen: &Screen, rows: u16, cols: u16) -> Vec<String> {
    (0..rows)
        .map(|row| render_visible_line(screen, row, cols))
        .collect()
}

fn render_visible_cells(screen: &Screen, rows: u16, cols: u16) -> Vec<Vec<TerminalCell>> {
    (0..rows)
        .map(|row| render_visible_row(screen, row, cols))
        .collect()
}

fn render_visible_line(screen: &Screen, row: u16, cols: u16) -> String {
    let mut line = String::new();
    for col in 0..cols {
        let Some(cell) = screen.cell(row, col) else {
            line.push(' ');
            continue;
        };

        if cell.is_wide_continuation() {
            continue;
        }

        if cell.has_contents() {
            line.push_str(&cell.contents());
        } else {
            line.push(' ');
        }
    }
    line
}

fn render_visible_row(screen: &Screen, row: u16, cols: u16) -> Vec<TerminalCell> {
    let mut rendered = Vec::with_capacity(cols as usize);

    for col in 0..cols {
        let Some(cell) = screen.cell(row, col) else {
            rendered.push(blank_cell(None, None));
            continue;
        };

        if cell.is_wide_continuation() {
            continue;
        }

        let mut foreground = color_to_hex(cell.fgcolor());
        let mut background = color_to_hex(cell.bgcolor());
        if cell.inverse() {
            std::mem::swap(&mut foreground, &mut background);
        }

        let text = if cell.has_contents() {
            cell.contents()
        } else {
            " ".to_owned()
        };

        rendered.push(TerminalCell {
            text,
            foreground,
            background,
            bold: cell.bold(),
            underline: cell.underline(),
        });
    }

    rendered
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

fn color_to_hex(color: Color) -> Option<u32> {
    match color {
        Color::Default => None,
        Color::Rgb(red, green, blue) => {
            Some((u32::from(red) << 16) | (u32::from(green) << 8) | u32::from(blue))
        }
        Color::Idx(index) => Some(ansi_index_to_hex(index)),
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

fn clamp_scrollback_offset(rows: usize, max_safe_rows: usize) -> usize {
    rows.min(max_safe_rows)
}

fn clamp_viewport_top(
    viewport_top: usize,
    total_lines: usize,
    viewport_height: usize,
) -> usize {
    viewport_top.min(total_lines.saturating_sub(viewport_height))
}

fn snapshot_viewport_from_parser(parser: &mut Parser) -> TerminalViewport {
    let current_scrollback = parser.screen().scrollback();
    parser.set_scrollback(usize::MAX);
    let max_scrollback = parser.screen().scrollback();
    parser.set_scrollback(0);

    let bottom_screen = parser.screen();
    let (rows, cols) = bottom_screen.size();
    let (cursor_row, _) = bottom_screen.cursor_position();
    let viewport_height = usize::from(rows);
    let total_lines = max_scrollback
        .saturating_add(visible_content_lines(bottom_screen, cols, cursor_row));

    parser.set_scrollback(current_scrollback);
    let viewport_top = total_lines
        .saturating_sub(viewport_height)
        .saturating_sub(current_scrollback);

    TerminalViewport {
        total_lines,
        viewport_top,
        viewport_height,
    }
}

fn visible_content_lines(screen: &Screen, cols: u16, cursor_row: u16) -> usize {
    let last_non_empty_line = screen
        .rows(0, cols)
        .enumerate()
        .filter_map(|(row_index, line)| {
            (!line.trim_end_matches(' ').is_empty()).then_some(row_index + 1)
        })
        .last()
        .unwrap_or(0);

    last_non_empty_line.max(usize::from(cursor_row).saturating_add(1))
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
        TERMINAL_PANEL_ID, TerminalViewport, ansi_index_to_hex, clamp_scrollback_offset,
        clamp_viewport_top, color_to_hex, snapshot_viewport_from_parser,
        terminal_panel_descriptor,
    };
    use panel::DockPlacement;
    use vt100::{Color, Parser};

    #[test]
    fn terminal_panel_defaults_to_bottom_singleton() {
        let descriptor = terminal_panel_descriptor();
        assert_eq!(descriptor.id.as_str(), TERMINAL_PANEL_ID);
        assert_eq!(descriptor.dock, DockPlacement::Bottom);
    }

    #[test]
    fn maps_terminal_palette_indexes_to_rgb_hex() {
        assert_eq!(color_to_hex(Color::Idx(1)), Some(0xcd3131));
        assert_eq!(ansi_index_to_hex(16), 0x000000);
        assert_eq!(ansi_index_to_hex(231), 0xffffff);
        assert_eq!(ansi_index_to_hex(244), 0x808080);
    }

    #[test]
    fn scrollback_offset_is_clamped_to_visible_row_count() {
        assert_eq!(clamp_scrollback_offset(0, 40), 0);
        assert_eq!(clamp_scrollback_offset(8, 40), 8);
        assert_eq!(clamp_scrollback_offset(120, 40), 40);
    }

    #[test]
    fn viewport_offset_is_clamped_to_total_history_not_viewport_height() {
        assert_eq!(clamp_viewport_top(0, 80, 20), 0);
        assert_eq!(clamp_viewport_top(12, 80, 20), 12);
        assert_eq!(clamp_viewport_top(75, 80, 20), 60);
    }

    #[test]
    fn jump_to_bottom_clears_viewport_offset() {
        let state = TerminalViewport {
            total_lines: 120,
            viewport_top: 40,
            viewport_height: 20,
        };

        assert_eq!(state.jump_to_bottom().viewport_top, 100);
        assert!(!state.jump_to_bottom().is_away_from_bottom());
    }

    #[test]
    fn resize_keeps_viewport_top_in_bounds() {
        let state = TerminalViewport {
            total_lines: 120,
            viewport_top: 95,
            viewport_height: 20,
        };

        assert_eq!(state.with_height(30).viewport_top, 90);
    }

    #[test]
    fn snapshot_total_lines_tracks_terminal_content_across_resize() {
        let mut parser = Parser::new(4, 20, 64);
        parser.process(b"one\r\ntwo\r\nthree");
        let initial = snapshot_viewport_from_parser(&mut parser);

        parser.set_size(8, 20);
        let resized = snapshot_viewport_from_parser(&mut parser);

        assert_eq!(initial.total_lines, 3);
        assert_eq!(resized.total_lines, 3);
        assert_eq!(resized.viewport_height, 8);
    }

    #[test]
    fn snapshot_total_lines_is_stable_when_scrolled_up() {
        let mut parser = Parser::new(3, 20, 64);
        parser.process(b"one\r\ntwo\r\nthree\r\nfour\r\nfive");
        let bottom = snapshot_viewport_from_parser(&mut parser);

        parser.set_scrollback(2);
        let scrolled_up = snapshot_viewport_from_parser(&mut parser);

        assert_eq!(bottom.total_lines, 5);
        assert_eq!(scrolled_up.total_lines, 5);
        assert_eq!(scrolled_up.viewport_top, 0);
    }

    #[test]
    fn snapshot_total_lines_does_not_double_count_scrollback_rows() {
        let mut parser = Parser::new(3, 20, 64);
        parser.process(b"one\r\ntwo\r\nthree\r\nfour\r\nfive\r\nsix");
        parser.process(b"\x1b[2J\x1b[H");
        let bottom = snapshot_viewport_from_parser(&mut parser);

        parser.set_scrollback(3);
        let scrolled_up = snapshot_viewport_from_parser(&mut parser);

        assert_eq!(bottom.total_lines, 4);
        assert_eq!(scrolled_up.total_lines, 4);
        assert_eq!(scrolled_up.viewport_top, 0);
    }
}
