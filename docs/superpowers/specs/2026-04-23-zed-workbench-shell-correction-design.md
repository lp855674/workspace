# Zed Workbench Shell Correction Design

## Goal

Fix the first workbench-kernel implementation so the base shell has real Zed-style window and dock behavior instead of a borderless content view.

This correction is focused on the missing foundation pieces:

- client-side title bar with real window control hit areas
- draggable title bar region
- top shell separated from workbench content
- bottom dock separated from status toolbar
- no debug/status placeholder text in the shell

## Problem

The current implementation configures:

- `WindowDecorations::Client`
- transparent `TitlebarOptions`

but it does not render a client-side title bar. That removes the native title bar while failing to replace it with Zed-style controls.

Current user-visible failures:

- no close, maximize, or minimize controls in the top-right corner
- no draggable title bar area
- bottom area is only a status bar, not bottom dock plus toolbar/status
- status bar shows debug strings such as focused panel and recent command count

## Reference

The repository already depends on `gpui-component = 0.5.1`, which provides `gpui_component::TitleBar`.

`TitleBar` already implements the required low-level window behavior:

- `WindowControlArea::Drag`
- `WindowControlArea::Min`
- `WindowControlArea::Max`
- `WindowControlArea::Close`
- `window.start_window_move()`
- `window.zoom_window()`
- platform-aware window controls for Windows/Linux

Therefore the correction should reuse `gpui_component::TitleBar` instead of hand-writing fake close/maximize/minimize buttons.

## Scope

### In Scope

- render `gpui_component::TitleBar` above the workbench shell
- use `TitleBar::title_bar_options()` for app window options
- keep `WindowDecorations::Client`
- make the title bar drag area real through `gpui_component::TitleBar`
- restructure shell rendering into:
  - title bar
  - main workbench body
  - bottom dock host
  - status toolbar
- render bottom dock only from `DockPlacement::Bottom`
- render status toolbar separately from bottom dock
- remove debug placeholder text from status toolbar

### Out of Scope

- full Zed title bar parity
- custom platform-specific title bar code copied from Zed
- tab drag-and-drop
- dock resizing
- pane splitting
- complete command palette UI

## Design

### Window Options

`app` should use:

- `titlebar: Some(gpui_component::TitleBar::title_bar_options())`
- `window_decorations: Some(WindowDecorations::Client)`
- `is_movable: true`
- current app id and bounds behavior

This keeps the window in client-decoration mode while giving the rendered root a matching title bar.

### App Frame

`app_ui::AppFrame` should become the composition owner for:

1. `TitleBar`
2. `WorkbenchShell`

The title bar is not a fake top row and must not contain fake menus or demo buttons. It can initially contain only stable identity text such as the app title.

### UI Shell Layout

`ui` should expose a shell model that separates bottom dock from status toolbar:

- `ShellTitle`
- `ShellSidebar`
- `ShellWorkspace`
- `ShellDockHost`
- `ShellStatusToolbar`

Rendering order:

1. title bar from `app_ui`
2. body row:
   - left dock/sidebar host
   - center dock host
   - optional right dock host
3. optional bottom dock host
4. status toolbar

The bottom dock host must not be embedded inside the center column as a status-like footer. It is a dock region controlled by `DockPlacement::Bottom`.

### Status Toolbar

The status toolbar is a fixed shell slot, but its contents are contribution-driven.

Default MVP state:

- no fake debug text
- no hardcoded `focused:` text
- no hardcoded `recent commands:` text
- render contribution items if present
- otherwise render an empty toolbar surface

### Empty Dock Host

An empty dock host may render no content or a neutral empty surface, but it must not explain itself as a placeholder feature.

Acceptable:

- blank content area
- active panel id while panel rendering is not yet wired

Not acceptable:

- fake logs
- fake inspector cards
- fake terminal output
- fake toolbar actions

## Acceptance Criteria

- The app window has visible close, maximize, and minimize controls on platforms where `gpui_component::TitleBar` renders them.
- The window can be dragged from the title bar.
- Double-click title bar behavior follows `gpui_component::TitleBar`.
- `app` uses `TitleBar::title_bar_options()` instead of manually incomplete title bar options.
- `app_ui` renders a real title bar above the workbench body.
- Bottom dock is a distinct dock host, not status bar content.
- Status toolbar is distinct from bottom dock.
- Status toolbar no longer renders `focused:` or `recent commands:` debug strings.
- Tests cover title bar composition and status toolbar empty default.

## Verification

Run:

- `cargo test -p app -p app_ui -p ui -- --nocapture`
- `cargo check -p app -p app_ui -p ui`
- `powershell -ExecutionPolicy Bypass -File .\script\verify.ps1`

## Rollback

If the title bar integration fails:

- keep `WindowDecorations::Client`
- temporarily render `TitleBar::new()` with no children
- preserve workbench body rendering
- do not return to fake window buttons or debug placeholder text
