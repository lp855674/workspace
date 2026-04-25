# Zed Terminal Viewport And Button Visibility Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the current double-scroll terminal behavior with a single terminal-owned viewport model and make title bar plus terminal header icon buttons visually clear.

**Architecture:** Keep the current crate boundaries. `terminal` becomes the source of truth for retained history and viewport slicing, `app_ui` maps keyboard and wheel input into viewport operations and preserves per-tab viewport state, and `ui` renders only the provided viewport while switching terminal controls to explicit button chrome.

**Tech Stack:** Rust 2024, `gpui`, `gpui-component`, `portable-pty`, `vt100`, workspace-local `terminal`, `app_ui`, `ui`, `app`.

---

## File Structure

- `crates/terminal/src/terminal.rs`
  - Add viewport metadata to `TerminalSnapshot`.
  - Replace the current visible-row-only scrollback clamp with total-history-aware viewport helpers.
  - Add tests for viewport clamping, jump-to-bottom, and resize-safe offsets.
- `crates/app_ui/src/app_ui.rs`
  - Stop clamping terminal scrollback to one viewport height.
  - Route wheel and PageUp/PageDown into terminal viewport operations.
  - Preserve per-tab viewport state and snap back to bottom before typing.
  - Add focused unit tests in the existing `#[cfg(test)]` module.
- `crates/ui/src/ui.rs`
  - Extend `ShellTerminalSession` with viewport metadata needed for rendering and button state.
  - Remove `.overflow_y_scrollbar()` from the terminal transcript surface.
  - Make terminal header actions and title bar controls render with explicit button affordance.
  - Add small render-shape helpers so tests can assert structure without UI screenshot testing.
- `crates/ui/tests/shell_layout_tests.rs`
  - Add tests for terminal viewport metadata, jump-to-bottom visibility, and dock host carriage.
- `crates/terminal/tech.md`
  - Update terminal responsibilities to say viewport slicing and retained-history bounds live in `terminal`.
- `tech.md`
  - Update runtime flow to say terminal scrolling is terminal-owned and UI no longer adds a second transcript scroll layer.

## Task 1: Make `terminal` the Single Viewport Source of Truth

**Files:**
- Modify: `crates/terminal/src/terminal.rs`

- [ ] **Step 1: Write the failing terminal snapshot tests**

Add these tests to the `#[cfg(test)]` module in `crates/terminal/src/terminal.rs`:

```rust
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
```

- [ ] **Step 2: Run the focused terminal test to verify it fails**

Run: `cargo test -p terminal viewport_offset_is_clamped_to_total_history_not_viewport_height -- --nocapture`

Expected: FAIL with unresolved items such as `clamp_viewport_top` or `TerminalViewport`.

- [ ] **Step 3: Add a terminal viewport model and snapshot metadata**

In `crates/terminal/src/terminal.rs`, add a small internal viewport type and extend the snapshot:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TerminalViewport {
    total_lines: usize,
    viewport_top: usize,
    viewport_height: usize,
}

impl TerminalViewport {
    fn max_viewport_top(&self) -> usize {
        self.total_lines.saturating_sub(self.viewport_height)
    }

    fn with_top(self, viewport_top: usize) -> Self {
        Self {
            viewport_top: clamp_viewport_top(
                viewport_top,
                self.total_lines,
                self.viewport_height,
            ),
            ..self
        }
    }

    fn with_height(self, viewport_height: usize) -> Self {
        Self {
            viewport_height,
            viewport_top: clamp_viewport_top(self.viewport_top, self.total_lines, viewport_height),
            ..self
        }
    }

    fn jump_to_bottom(self) -> Self {
        self.with_top(self.max_viewport_top())
    }

    fn is_away_from_bottom(&self) -> bool {
        self.viewport_top < self.max_viewport_top()
    }
}

fn clamp_viewport_top(viewport_top: usize, total_lines: usize, viewport_height: usize) -> usize {
    viewport_top.min(total_lines.saturating_sub(viewport_height))
}
```

Extend `TerminalSnapshot` with:

```rust
pub struct TerminalSnapshot {
    // existing fields...
    pub total_lines: usize,
    pub viewport_top: usize,
    pub viewport_height: usize,
    pub can_scroll_up: bool,
    pub can_scroll_down: bool,
}
```

- [ ] **Step 4: Replace the visible-row clamp with total-history-aware viewport helpers**

Keep the current `vt100` parser, but stop using viewport height as the max history range. In `TerminalSession`, replace:

```rust
pub fn max_safe_scrollback(&self) -> usize {
    let (rows, _) = self.parser.screen().size();
    usize::from(rows)
}
```

with a history-aware helper:

```rust
pub fn total_lines(&self) -> usize {
    usize::from(self.parser.screen().rows()) + self.parser.screen().scrollback()
}

pub fn viewport_height(&self) -> usize {
    let (rows, _) = self.parser.screen().size();
    usize::from(rows)
}

pub fn max_viewport_top(&self) -> usize {
    self.total_lines().saturating_sub(self.viewport_height())
}

pub fn set_viewport_top(&mut self, viewport_top: usize) {
    let max_viewport_top = self.max_viewport_top();
    let clamped_top = viewport_top.min(max_viewport_top);
    let scrollback = max_viewport_top.saturating_sub(clamped_top);
    self.parser.set_scrollback(scrollback);
}
```

Then build snapshot metadata from:

```rust
let total_lines = self.total_lines();
let viewport_height = usize::from(rows);
let viewport_top = total_lines
    .saturating_sub(viewport_height)
    .saturating_sub(screen.scrollback());
```

- [ ] **Step 5: Run the terminal crate tests**

Run: `cargo test -p terminal -- --nocapture`

Expected: PASS, including the new viewport tests and the existing color/panel tests.

- [ ] **Step 6: Commit**

```bash
git add crates/terminal/src/terminal.rs
git commit -m "feat: make terminal own viewport state"
```

## Task 2: Rewire `app_ui` to Use Terminal Viewport Operations

**Files:**
- Modify: `crates/app_ui/src/app_ui.rs`

- [ ] **Step 1: Write the failing app frame tests**

Add these tests to the existing `#[cfg(test)]` module in `crates/app_ui/src/app_ui.rs`:

```rust
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
```

- [ ] **Step 2: Run the focused `app_ui` test to verify it fails**

Run: `cargo test -p app_ui page_navigation_uses_total_history_bounds -- --nocapture`

Expected: FAIL with unresolved helpers such as `clamp_terminal_viewport_top`.

- [ ] **Step 3: Switch tab runtime state from `viewport_scrollback` to `viewport_top`**

In `crates/app_ui/src/app_ui.rs`, replace:

```rust
struct TerminalTabRuntime {
    id: String,
    label: String,
    session: TerminalSession,
    viewport_scrollback: usize,
}
```

with:

```rust
struct TerminalTabRuntime {
    id: String,
    label: String,
    session: TerminalSession,
    viewport_top: usize,
}
```

When building `ShellTerminalSession`, pass through the terminal snapshot metadata:

```rust
ShellTerminalSession {
    // existing fields...
    viewport_top: snapshot.viewport_top,
    viewport_height: snapshot.viewport_height,
    total_lines: snapshot.total_lines,
    can_scroll_up: snapshot.can_scroll_up,
    can_scroll_down: snapshot.can_scroll_down,
    away_from_bottom: snapshot.can_scroll_down,
}
```

- [ ] **Step 4: Replace scrollback math with viewport-top math**

In `scroll_active_terminal`, replace the current one-screen clamp:

```rust
let max_scrollback = tab.session.max_safe_scrollback().min(visible_rows);
let clamped = next.min(max_scrollback);
tab.viewport_scrollback = clamped;
tab.session.set_scrollback(clamped);
```

with viewport-top movement:

```rust
let viewport_height = visible_rows;
let live_bottom_top = tab.session.total_lines().saturating_sub(viewport_height);
let current_top = tab.viewport_top.min(live_bottom_top);

if lines == i32::MIN {
    tab.viewport_top = live_bottom_top;
    tab.session.set_viewport_top(live_bottom_top);
    return;
}

let next_top = if lines >= 0 {
    current_top.saturating_sub(lines as usize)
} else {
    (current_top + (-lines) as usize).min(live_bottom_top)
};

tab.viewport_top = next_top;
tab.session.set_viewport_top(next_top);
```

Before writing terminal input in `handle_terminal_key_event`, snap to bottom:

```rust
let live_bottom_top = tab
    .session
    .total_lines()
    .saturating_sub(self.visible_terminal_rows());
tab.viewport_top = live_bottom_top;
tab.session.set_viewport_top(live_bottom_top);
```

- [ ] **Step 5: Preserve viewport per tab and after resize**

Update tab activation and resize paths:

```rust
if let Some(tab) = self.active_terminal_mut() {
    let visible_rows = self.visible_terminal_rows();
    let live_bottom_top = tab.session.total_lines().saturating_sub(visible_rows);
    tab.viewport_top = tab.viewport_top.min(live_bottom_top);
    tab.session.set_viewport_top(tab.viewport_top);
}
```

Also initialize new tabs at the live bottom:

```rust
self.terminal_tabs.push(TerminalTabRuntime {
    id: id.clone(),
    label,
    session,
    viewport_top: 0,
});
```

Then immediately normalize with:

```rust
if let Some(tab) = self.active_terminal_mut() {
    let live_bottom_top = tab
        .session
        .total_lines()
        .saturating_sub(self.visible_terminal_rows());
    tab.viewport_top = live_bottom_top;
    tab.session.set_viewport_top(live_bottom_top);
}
```

- [ ] **Step 6: Run focused `app_ui` verification**

Run: `cargo test -p app_ui -- --nocapture`

Expected: PASS with the new viewport helper tests.

- [ ] **Step 7: Commit**

```bash
git add crates/app_ui/src/app_ui.rs
git commit -m "fix: route app frame scrolling through terminal viewport"
```

## Task 3: Remove Native Transcript Scrolling and Clarify Buttons in `ui`

**Files:**
- Modify: `crates/ui/src/ui.rs`
- Modify: `crates/ui/tests/shell_layout_tests.rs`

- [ ] **Step 1: Write the failing UI model tests**

Add these tests to `crates/ui/tests/shell_layout_tests.rs`:

```rust
#[test]
fn terminal_session_carries_viewport_metadata() {
    let terminal = ShellTerminalSession {
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
        scrollback: 0,
        viewport_top: 80,
        viewport_height: 20,
        total_lines: 100,
        can_scroll_up: true,
        can_scroll_down: false,
        away_from_bottom: false,
        visible_cells: vec![],
        visible_lines: vec![],
    };

    assert_eq!(terminal.viewport_top, 80);
    assert_eq!(terminal.total_lines, 100);
    assert!(terminal.can_scroll_up);
    assert!(!terminal.can_scroll_down);
}

#[test]
fn jump_to_bottom_badge_only_shows_when_terminal_is_away_from_bottom() {
    let hidden = ShellTerminalSession {
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
        scrollback: 0,
        viewport_top: 80,
        viewport_height: 20,
        total_lines: 100,
        can_scroll_up: true,
        can_scroll_down: false,
        away_from_bottom: false,
        visible_cells: vec![],
        visible_lines: vec![],
    };

    let shown = ShellTerminalSession {
        away_from_bottom: true,
        can_scroll_down: true,
        scrollback: 12,
        ..hidden.clone()
    };

    assert!(!hidden.away_from_bottom);
    assert!(shown.away_from_bottom);
    assert!(shown.can_scroll_down);
}
```

- [ ] **Step 2: Run the focused UI test to verify it fails**

Run: `cargo test -p ui terminal_session_carries_viewport_metadata -- --nocapture`

Expected: FAIL because the new `ShellTerminalSession` fields do not exist yet.

- [ ] **Step 3: Extend the UI terminal model and remove native transcript scrolling**

In `crates/ui/src/ui.rs`, extend the session model:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShellTerminalSession {
    // existing fields...
    pub viewport_top: usize,
    pub viewport_height: usize,
    pub total_lines: usize,
    pub can_scroll_up: bool,
    pub can_scroll_down: bool,
}
```

Then change `render_terminal_session` from:

```rust
div()
    .size_full()
    .overflow_y_scrollbar()
    .font_family("Consolas")
```

to:

```rust
div()
    .size_full()
    .font_family("Consolas")
```

Keep the inner fixed viewport:

```rust
div()
    .size_full()
    .px_3()
    .py_2()
    .pr_5()
    .child(div().w_full().flex().flex_col().children(output_lines))
```

- [ ] **Step 4: Make terminal header and title bar controls explicit buttons**

Refine `header_button` and `window_control_button` in `crates/ui/src/ui.rs`:

```rust
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
        .h(px(28.))
        .w(px(28.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(5.))
        .border_1()
        .border_color(rgb(theme.border))
        .bg(rgb(theme.status_bar_button))
        .text_color(rgb(theme.status_bar_active))
        .hover(move |style| {
            style
                .bg(rgb(theme.status_bar_button_hover))
                .text_color(rgb(0xffffff))
        })
        .tooltip(move |window, cx| Tooltip::new(tooltip).build(window, cx))
        .on_click(move |_, window, cx| on_action(&action, window, cx))
        .child(Icon::new(icon).small())
}
```

For title bar controls in `crates/app_ui/src/app_ui.rs`, keep `WindowControlArea` but strengthen visibility:

```rust
div()
    .id(id)
    .h_full()
    .w(px(46.))
    .flex()
    .items_center()
    .justify_center()
    .bg(rgb(theme.title_bar_background))
    .text_color(rgb(theme.window_control_foreground))
    .hover(move |style| {
        if is_close {
            style.bg(rgb(theme.window_control_close_hover)).text_color(rgb(0xffffff))
        } else {
            style.bg(rgb(theme.window_control_hover)).text_color(rgb(0xffffff))
        }
    })
    .window_control_area(control_area)
    .child(Icon::new(icon).small())
```

- [ ] **Step 5: Update the jump-to-bottom badge to use viewport state**

In `render_terminal_header_actions`, use:

```rust
.when_some(
    terminal.filter(|terminal| terminal.away_from_bottom && terminal.can_scroll_down),
    |this, terminal| {
        this.child(
            div()
                .h(px(28.))
                .px_2()
                .flex()
                .items_center()
                .gap_1()
                .rounded(px(5.))
                .border_1()
                .border_color(rgb(theme.border))
                .bg(rgb(theme.status_bar_button_active))
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
```

- [ ] **Step 6: Run UI verification**

Run: `cargo test -p ui -- --nocapture`

Expected: PASS, including the updated shell layout tests.

- [ ] **Step 7: Commit**

```bash
git add crates/ui/src/ui.rs crates/ui/tests/shell_layout_tests.rs crates/app_ui/src/app_ui.rs
git commit -m "fix: remove double scrolling and clarify terminal controls"
```

## Task 4: Sync Docs and Run Full Verification

**Files:**
- Modify: `crates/terminal/tech.md`
- Modify: `tech.md`

- [ ] **Step 1: Update terminal ownership docs**

In `crates/terminal/tech.md`, update responsibilities to say:

```md
- maintain PTY-backed terminal history and viewport bounds
- expose a viewport-aware snapshot for `app_ui` / `ui`, including terminal cells, ANSI styles, cursor state, retained history size, and viewport position
- allow the workbench runtime to move the terminal viewport through scrollback without adding a second UI scroll model
```

- [ ] **Step 2: Update workspace runtime docs**

In `tech.md`, update `Runtime Flow` with:

```md
7. `ui` renders the terminal as a fixed viewport from runtime state instead of applying an additional native transcript scroll container.
8. `app_ui` maps wheel and page navigation input into terminal viewport operations and preserves per-tab terminal viewport state.
```

- [ ] **Step 3: Run focused crate verification**

Run:

```powershell
cargo test -p terminal -p ui -p app_ui -- --nocapture
cargo check -p terminal -p ui -p app_ui
```

Expected: PASS.

- [ ] **Step 4: Run repository verification**

Run:

```powershell
powershell -ExecutionPolicy Bypass -File .\script\verify.ps1
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/terminal/tech.md tech.md
git commit -m "docs: sync terminal viewport behavior"
```

## Self-Review

- Spec coverage: Task 1 covers terminal-owned viewport state and history bounds. Task 2 covers wheel/key input, tab preservation, and snap-to-bottom-on-input. Task 3 covers UI removal of native transcript scrolling plus visible title bar and terminal action buttons. Task 4 covers the required docs and verification.
- Placeholder scan: no task uses TBD/TODO, “appropriate handling,” or indirect references without code.
- Type consistency: the plan consistently uses `viewport_top`, `viewport_height`, `total_lines`, `can_scroll_up`, and `can_scroll_down` across `terminal`, `app_ui`, and `ui`.
- Open risk tracked from spec: if `vt100` cannot supply enough retained-history detail, Task 1 should introduce a thin internal history helper inside `crates/terminal/src/terminal.rs` rather than reintroducing UI-owned scrolling.
