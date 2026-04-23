# Technical Specification

## Goal

This workspace provides a Zed-style workbench kernel in Rust. It keeps shell control, dock state, panel lifecycle, typed actions, and persistence in shared crates so feature modules attach through stable contributions instead of owning layout or window behavior.

## Architecture

- `app` owns process startup, native menu installation, client-side window setup, and runtime bootstrap.
- `app_ui` owns the root frame model and renders shell hosts from `workspace` runtime state.
- `workspace` owns panel registration, singleton instance lifecycle, dock layout mutation, focus transitions, command history, and restore-ready serialization.
- `module` owns feature-module contracts and contribution conflict detection.
- `panel` owns panel type ids, panel instance ids, singleton lifetime rules, and default close behavior.
- `dock` owns left/center/right/bottom dock containers, active tabs, and visibility state.
- `actions`, `commands`, `menu`, `keymap`, and `command_palette` share one typed action model that dispatches into `workspace`.
- `status_bar` and `notifications` own contribution surfaces; they do not own workbench layout.
- `settings`, `theme`, `keymap`, `paths`, and `assets` own editable configuration and bundled defaults.
- `db` owns `sqlx + sqlite` migration text and SQLite connection helpers.
- `welcome` is the first built-in singleton panel used to prove the contribution path.

## Dependency Policy

- `gpui = "0.2.2"`
- `gpui_macros = { package = "gpui-macros", version = "0.2.2" }`
- `gpui-component = "0.5.1"`
- `sqlx = "0.8"` with SQLite runtime support
- Workspace crates remain unpublished and depend on each other by local path only inside the workspace.
- Host concerns stay isolated by crate boundary; only `crates/db` may use `sqlx` directly.

## Runtime Flow

1. `app` boots the Zed-style shell with client-side window decorations, installs the native menu through `gpui`, and uses `gpui_component::TitleBar::title_bar_options()` for titlebar compatibility.
2. `module::ModuleRuntime` registers built-in modules and validates command/panel contribution conflicts.
3. `commands`, menu entries, keymaps, and command-palette surfaces carry typed `actions::ActionEnvelope` values.
4. `workspace::WorkspaceController` is the runtime control point for opening, toggling, focusing, hiding, restoring, and serializing panels.
5. `dock::DockLayoutState` records which panel instance is attached to each dock and which tab is active.
6. `app_ui` renders `gpui_component::TitleBar` above the workbench body, so drag and window controls are platform hit areas rather than fake buttons.
7. `ui` renders dock hosts from runtime state and keeps the bottom dock host separate from the status toolbar.
8. The status toolbar renders contribution items only and does not render debug placeholder strings.
9. `welcome` contributes the initial singleton center panel and command action used to validate the full open/focus/serialize path.

## Persistence

- User-editable files:
  - `settings.json`
  - `keymap.json`
  - bundled theme JSON under `assets/themes`
- SQLite database:
  - `workspace_sessions`
  - `panel_states`
  - `command_history`
  - `notifications`
  - `module_state`
- Workspace shell sessions use schema version `2`.
- Restore must tolerate unknown panel types, removed panels, incompatible state blobs, and old schema versions by skipping invalid records instead of blocking startup.

## Rollback

- Revert the workbench-kernel commits back to the last fake-shell baseline if shell bootstrap regresses.
- Keep `0002_workspace_shell_session.sql` present but skip runtime restore reads if session restore causes startup failures.
- Fall back to an empty default dock layout with restore disabled while preserving registered panel descriptors.

## Verification

- `cargo test -p db`
- `cargo check -p db`
- `cargo test -p panel -p dock -p workspace -p actions -p commands -p welcome -p db`
- `cargo test -p workspace -p welcome -p app`
- `cargo check --workspace`
- `cargo check -p ui -p app_ui -p app`
- `cargo test -p ui -p app_ui -p app`
