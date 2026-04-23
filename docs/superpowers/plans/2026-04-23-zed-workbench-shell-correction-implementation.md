# Zed Workbench Shell Correction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the missing real Zed-style titlebar/window controls and split bottom dock from the status toolbar.

**Architecture:** Reuse `gpui_component::TitleBar` instead of hand-writing fake window controls. Keep `app` responsible for client-decoration window options, keep `app_ui` as the root composition owner, and keep `ui` as layout rendering helpers for body, bottom dock, and status toolbar.

**Tech Stack:** Rust 2024, `gpui`, `gpui-component`, workspace-local `app`, `app_ui`, `ui`, `dock`, `workspace`, `status_bar`.

---

## File Structure

- `crates/app/src/main.rs`
  - Use `gpui_component::TitleBar::title_bar_options()` for titlebar configuration.
  - Keep `WindowDecorations::Client` and set `is_movable: true`.
- `crates/app/tests/startup_smoke.rs`
  - Verify titlebar options come from the shell helper and are transparent/client-titlebar compatible.
- `crates/app_ui/src/app_ui.rs`
  - Compose `TitleBar` above the workbench body.
  - Stop sending debug status text.
  - Pass bottom dock separately from main workspace body.
- `crates/app/tests/bootstrap_runtime_tests.rs`
  - Verify the frame starts with empty status toolbar text and titlebar-enabled shell.
- `crates/ui/src/ui.rs`
  - Split rendering into main body, optional bottom dock, and status toolbar.
  - Replace `ShellStatus` text fields with contribution-only toolbar items.
- `crates/ui/tests/shell_layout_tests.rs`
  - Add data-shape tests proving bottom dock and status toolbar are separate.
- `tech.md`
  - Update runtime flow to say titlebar/window controls are powered by `gpui_component::TitleBar`.

## Task 1: Use the Real GPUI Component TitleBar for Window Options

**Files:**
- Modify: `crates/app/src/main.rs`
- Modify: `crates/app/tests/startup_smoke.rs`

- [ ] **Step 1: Write the failing test**

Add this test to `crates/app/tests/startup_smoke.rs`:

```rust
use gpui_component::TitleBar;

#[test]
fn app_uses_gpui_component_title_bar_options() {
    let options = TitleBar::title_bar_options();

    assert!(options.appears_transparent);
    assert!(options.title.is_none());
    assert!(options.traffic_light_position.is_some());
}
```

- [ ] **Step 2: Run the focused test to verify the dependency is available**

Run: `cargo test -p app app_uses_gpui_component_title_bar_options -- --nocapture`

Expected: PASS if `gpui_component::TitleBar` is available to the `app` crate; otherwise FAIL with an unresolved import.

- [ ] **Step 3: Replace manual titlebar options**

In `crates/app/src/main.rs`, change imports and `WindowOptions`:

```rust
use gpui::{
    Action, App, AppContext, Application, Menu, MenuItem, OsAction, WindowBounds,
    WindowDecorations, WindowOptions, actions, px, size,
};
use gpui_component::{Root, TitleBar};
```

Use:

```rust
WindowOptions {
    window_bounds: Some(window_bounds),
    titlebar: Some(TitleBar::title_bar_options()),
    app_id: Some("com.workspace.zed_workbench_kernel".to_owned()),
    window_decorations: Some(WindowDecorations::Client),
    is_movable: true,
    focus: true,
    ..Default::default()
}
```

- [ ] **Step 4: Run the app crate tests**

Run: `cargo test -p app -- --nocapture`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/app/src/main.rs crates/app/tests/startup_smoke.rs
git commit -m "fix: use real titlebar window options"
```

## Task 2: Render TitleBar Above the Workbench Body

**Files:**
- Modify: `crates/app_ui/src/app_ui.rs`
- Modify: `crates/app/tests/bootstrap_runtime_tests.rs`

- [ ] **Step 1: Write the failing test**

Add this test to `crates/app/tests/bootstrap_runtime_tests.rs`:

```rust
#[test]
fn app_frame_starts_with_empty_status_toolbar_text() {
    let frame = AppFrame::new(WorkspaceController::new("session-1"));

    assert_eq!(frame.status_toolbar_text(), "");
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p app app_frame_starts_with_empty_status_toolbar_text -- --nocapture`

Expected: FAIL because `status_toolbar_text` does not exist.

- [ ] **Step 3: Add titlebar-aware frame composition**

In `crates/app_ui/src/app_ui.rs`, import `TitleBar` and expose an empty status toolbar text helper:

```rust
use gpui_component::{
    TitleBar,
    tree::{TreeItem, TreeState},
};
```

Add:

```rust
pub fn status_toolbar_text(&self) -> &'static str {
    ""
}
```

Change `render` to wrap the shell with a real titlebar:

```rust
gpui::div()
    .size_full()
    .flex()
    .flex_col()
    .child(
        TitleBar::new().child(
            gpui::div()
                .flex()
                .items_center()
                .text_sm()
                .child(self.title),
        ),
    )
    .child(render_shell(sidebar, workspace, status))
```

Set status to contribution-only:

```rust
let status = ShellStatusToolbar {
    items: self.status_items.clone(),
};
```

- [ ] **Step 4: Run app and app_ui tests**

Run: `cargo test -p app -p app_ui -- --nocapture`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/app_ui/src/app_ui.rs crates/app/tests/bootstrap_runtime_tests.rs
git commit -m "fix: render real titlebar in app frame"
```

## Task 3: Split Bottom Dock From Status Toolbar

**Files:**
- Modify: `crates/ui/src/ui.rs`
- Create: `crates/ui/tests/shell_layout_tests.rs`
- Modify: `crates/app_ui/src/app_ui.rs`

- [ ] **Step 1: Write the failing UI model tests**

Create `crates/ui/tests/shell_layout_tests.rs`:

```rust
use status_bar::StatusBarItem;
use ui::{ShellDockHost, ShellStatusToolbar, ShellWorkspace};

#[test]
fn shell_workspace_models_bottom_dock_separately() {
    let workspace = ShellWorkspace {
        center: ShellDockHost::new("dock.center", "Center", Some("welcome.panel".to_owned()), true),
        right: ShellDockHost::new("dock.right", "Right Dock", None, false),
        bottom: ShellDockHost::new("dock.bottom", "Bottom Dock", Some("terminal.panel".to_owned()), true),
    };

    assert_eq!(workspace.bottom.id, "dock.bottom");
    assert!(workspace.bottom.visible);
    assert_eq!(workspace.bottom.active_panel.as_deref(), Some("terminal.panel"));
}

#[test]
fn status_toolbar_defaults_to_contribution_items_only() {
    let toolbar = ShellStatusToolbar {
        items: Vec::<StatusBarItem>::new(),
    };

    assert!(toolbar.items.is_empty());
}
```

- [ ] **Step 2: Run the UI test to verify it fails**

Run: `cargo test -p ui shell_workspace_models_bottom_dock_separately -- --nocapture`

Expected: FAIL because `ShellDockHost::new` and `ShellStatusToolbar` do not exist.

- [ ] **Step 3: Add shell model constructors and toolbar type**

In `crates/ui/src/ui.rs`, replace `ShellStatus` with:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShellStatusToolbar {
    pub items: Vec<StatusBarItem>,
}
```

Add to `ShellDockHost`:

```rust
impl ShellDockHost {
    pub const fn new(
        id: &'static str,
        title: &'static str,
        active_panel: Option<String>,
        visible: bool,
    ) -> Self {
        Self {
            id,
            title,
            active_panel,
            visible,
        }
    }
}
```

Change `render_shell` signature:

```rust
pub fn render_shell(
    sidebar: ShellSidebar,
    workspace: ShellWorkspace,
    status: ShellStatusToolbar,
) -> impl IntoElement
```

Render body and bottom dock separately:

```rust
.child(
    div()
        .flex()
        .flex_1()
        .min_h_0()
        .child(render_sidebar(sidebar))
        .child(render_workspace_body(&workspace)),
)
.when(workspace.bottom.visible, |this| {
    this.child(render_dock_host(workspace.bottom, DockHostKind::Bottom))
})
.child(render_status_toolbar(status))
```

Rename the old status renderer to `render_status_toolbar` and render only contribution items:

```rust
fn render_status_toolbar(status: ShellStatusToolbar) -> impl IntoElement {
    div()
        .h(px(24.))
        .w_full()
        .px_3()
        .flex()
        .items_center()
        .justify_between()
        .bg(rgb(0x0c1116))
        .border_t_1()
        .border_color(rgb(0x1b242d))
        .children(status.items.into_iter().map(|item| {
            div()
                .text_xs()
                .text_color(rgb(0x8092a2))
                .child(item.text)
        }))
}
```

- [ ] **Step 4: Update app_ui to use the new UI model**

In `crates/app_ui/src/app_ui.rs`, replace `ShellStatus` imports/usages with `ShellStatusToolbar`:

```rust
use ui::{
    ShellDockHost, ShellSidebar, ShellStatusToolbar, ShellWorkspace, render_file_tree, render_shell,
};
```

Build dock hosts with constructors:

```rust
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
    workspace_state.dock_layout.is_visible(DockPlacement::Bottom),
),
```

Use:

```rust
let status = ShellStatusToolbar {
    items: self.status_items.clone(),
};
```

- [ ] **Step 5: Run focused shell tests**

Run: `cargo test -p ui -p app_ui -p app -- --nocapture`

Expected: PASS with no `focused:` or `recent commands:` status text.

- [ ] **Step 6: Commit**

```bash
git add crates/ui/src/ui.rs crates/ui/tests/shell_layout_tests.rs crates/app_ui/src/app_ui.rs
git commit -m "fix: split bottom dock from status toolbar"
```

## Task 4: Sync Documentation and Verify

**Files:**
- Modify: `tech.md`

- [ ] **Step 1: Update `tech.md` runtime flow**

Add these points under `Runtime Flow`:

```md
1. `app` uses `gpui_component::TitleBar::title_bar_options()` with client-side decorations.
2. `app_ui` renders `gpui_component::TitleBar` above the workbench body, so drag and window controls are platform hit areas rather than fake buttons.
3. `ui` separates the bottom dock host from the status toolbar.
4. The status toolbar renders contribution items only and does not render debug placeholder strings.
```

- [ ] **Step 2: Run verification**

Run:

```powershell
powershell -ExecutionPolicy Bypass -File .\script\verify.ps1
```

Expected: PASS.

- [ ] **Step 3: Run focused verification**

Run:

```powershell
$env:CARGO_TARGET_DIR='C:\Users\Hi\.codex\memories\target-shell-correction'; cargo test -p app -p app_ui -p ui -- --nocapture
```

Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add tech.md
git commit -m "docs: sync titlebar and dock shell behavior"
```

## Self-Review

- Spec coverage: all shell correction requirements are covered by Tasks 1-4.
- Placeholder scan: no task uses TBD/TODO placeholders.
- Type consistency: `ShellStatusToolbar`, `ShellDockHost::new`, and `TitleBar` are introduced before later usage.
- Risk: `gpui_component::TitleBar` visual behavior is platform-implemented and cannot be fully asserted in headless unit tests; tests cover options and shell composition data, while runtime behavior relies on GPUI's existing `WindowControlArea` implementation.
