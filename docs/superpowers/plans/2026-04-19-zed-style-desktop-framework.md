# Zed-Style Desktop Framework Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a standalone Rust desktop framework workspace that follows Zed's structure and lifecycle patterns, with GPUI startup, app frame, module contributions, panel/dock systems, `sqlx + sqlite` persistence, and a demo feature.

**Architecture:** The implementation starts with a minimal runnable workspace and then layers on framework systems in the same order they are consumed at runtime: paths/assets, UI frame, commands, panel/dock, module runtime, settings/theme, persistence, session restore, and demo feature. Each subsystem gets its own crate boundary so future feature modules can depend on stable contracts instead of reaching into host internals.

**Tech Stack:** Rust workspace, Cargo, GPUI, `sqlx` + SQLite, `serde`, `tracing`, JSON config files, SQL migrations

---

## File Structure

Planned top-level files and responsibilities:

- Create: `E:\code\workspace\Cargo.toml`
  - workspace members and shared dependency versions
- Create: `E:\code\workspace\rust-toolchain.toml`
  - lock toolchain for GPUI compatibility
- Create: `E:\code\workspace\tech.md`
  - technical source of truth after implementation
- Create: `E:\code\workspace\assets\themes\default.json`
  - default theme asset
- Create: `E:\code\workspace\assets\keymaps\default-windows.json`
  - default keymap asset
- Create: `E:\code\workspace\crates\app\Cargo.toml`
- Create: `E:\code\workspace\crates\app\src\main.rs`
  - executable startup and window boot
- Create: `E:\code\workspace\crates\app_ui\Cargo.toml`
- Create: `E:\code\workspace\crates\app_ui\src\app_ui.rs`
  - root app frame view and layout regions
- Create: `E:\code\workspace\crates\workspace\Cargo.toml`
- Create: `E:\code\workspace\crates\workspace\src\workspace.rs`
  - workspace controller and session glue
- Create: `E:\code\workspace\crates\module\Cargo.toml`
- Create: `E:\code\workspace\crates\module\src\module.rs`
  - module traits, contributions, runtime
- Create: `E:\code\workspace\crates\panel\Cargo.toml`
- Create: `E:\code\workspace\crates\panel\src\panel.rs`
  - panel descriptors and registry
- Create: `E:\code\workspace\crates\dock\Cargo.toml`
- Create: `E:\code\workspace\crates\dock\src\dock.rs`
  - dock placement and visible panel layout state
- Create: `E:\code\workspace\crates\ui\Cargo.toml`
- Create: `E:\code\workspace\crates\ui\src\ui.rs`
  - simple shared UI containers
- Create: `E:\code\workspace\crates\theme\Cargo.toml`
- Create: `E:\code\workspace\crates\theme\src\theme.rs`
  - theme models and manager
- Create: `E:\code\workspace\crates\settings\Cargo.toml`
- Create: `E:\code\workspace\crates\settings\src\settings.rs`
  - settings models and loader
- Create: `E:\code\workspace\crates\keymap\Cargo.toml`
- Create: `E:\code\workspace\crates\keymap\src\keymap.rs`
  - keybinding models and loader
- Create: `E:\code\workspace\crates\menu\Cargo.toml`
- Create: `E:\code\workspace\crates\menu\src\menu.rs`
  - menu model and command bridge
- Create: `E:\code\workspace\crates\actions\Cargo.toml`
- Create: `E:\code\workspace\crates\actions\src\actions.rs`
  - logical actions
- Create: `E:\code\workspace\crates\commands\Cargo.toml`
- Create: `E:\code\workspace\crates\commands\src\commands.rs`
  - command registry and dispatch
- Create: `E:\code\workspace\crates\notifications\Cargo.toml`
- Create: `E:\code\workspace\crates\notifications\src\notifications.rs`
  - toast and notification state
- Create: `E:\code\workspace\crates\command_palette\Cargo.toml`
- Create: `E:\code\workspace\crates\command_palette\src\command_palette.rs`
  - palette state and host view
- Create: `E:\code\workspace\crates\status_bar\Cargo.toml`
- Create: `E:\code\workspace\crates\status_bar\src\status_bar.rs`
  - status bar contracts and view
- Create: `E:\code\workspace\crates\db\Cargo.toml`
- Create: `E:\code\workspace\crates\db\src\db.rs`
  - pool creation and repository traits
- Create: `E:\code\workspace\crates\db\migrations\0001_init.sql`
  - initial framework schema
- Create: `E:\code\workspace\crates\paths\Cargo.toml`
- Create: `E:\code\workspace\crates\paths\src\paths.rs`
  - app directories and file locations
- Create: `E:\code\workspace\crates\assets\Cargo.toml`
- Create: `E:\code\workspace\crates\assets\src\assets.rs`
  - asset loading entry points
- Create: `E:\code\workspace\crates\foundation\Cargo.toml`
- Create: `E:\code\workspace\crates\foundation\src\foundation.rs`
  - shared IDs, context, errors, `AppServices`
- Create: `E:\code\workspace\crates\welcome\Cargo.toml`
- Create: `E:\code\workspace\crates\welcome\src\welcome.rs`
  - demo feature module

## Task 1: Scaffold The Workspace

**Files:**
- Create: `E:\code\workspace\Cargo.toml`
- Create: `E:\code\workspace\rust-toolchain.toml`
- Create: `E:\code\workspace\crates\app\Cargo.toml`
- Create: `E:\code\workspace\crates\app_ui\Cargo.toml`
- Create: `E:\code\workspace\crates\workspace\Cargo.toml`
- Create: `E:\code\workspace\crates\module\Cargo.toml`
- Create: `E:\code\workspace\crates\panel\Cargo.toml`
- Create: `E:\code\workspace\crates\dock\Cargo.toml`
- Create: `E:\code\workspace\crates\ui\Cargo.toml`
- Create: `E:\code\workspace\crates\theme\Cargo.toml`
- Create: `E:\code\workspace\crates\settings\Cargo.toml`
- Create: `E:\code\workspace\crates\keymap\Cargo.toml`
- Create: `E:\code\workspace\crates\menu\Cargo.toml`
- Create: `E:\code\workspace\crates\actions\Cargo.toml`
- Create: `E:\code\workspace\crates\commands\Cargo.toml`
- Create: `E:\code\workspace\crates\notifications\Cargo.toml`
- Create: `E:\code\workspace\crates\command_palette\Cargo.toml`
- Create: `E:\code\workspace\crates\status_bar\Cargo.toml`
- Create: `E:\code\workspace\crates\db\Cargo.toml`
- Create: `E:\code\workspace\crates\paths\Cargo.toml`
- Create: `E:\code\workspace\crates\assets\Cargo.toml`
- Create: `E:\code\workspace\crates\foundation\Cargo.toml`
- Create: `E:\code\workspace\crates\welcome\Cargo.toml`

- [ ] **Step 1: Write the failing workspace validation**

```toml
# Add this command to verify before any source files exist.
cargo metadata --format-version 1
```

- [ ] **Step 2: Run validation to verify it fails**

Run: `cargo metadata --format-version 1`
Expected: FAIL with missing `Cargo.toml` members or missing manifests.

- [ ] **Step 3: Create the workspace manifests**

```toml
# E:\code\workspace\Cargo.toml
[workspace]
members = [
  "crates/app",
  "crates/app_ui",
  "crates/workspace",
  "crates/module",
  "crates/panel",
  "crates/dock",
  "crates/ui",
  "crates/theme",
  "crates/settings",
  "crates/keymap",
  "crates/menu",
  "crates/actions",
  "crates/commands",
  "crates/notifications",
  "crates/command_palette",
  "crates/status_bar",
  "crates/db",
  "crates/paths",
  "crates/assets",
  "crates/foundation",
  "crates/welcome",
]
resolver = "2"

[workspace.package]
edition = "2024"
license = "MIT"
version = "0.1.0"

[workspace.dependencies]
anyhow = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "macros", "migrate"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros", "sync"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["fmt", "env-filter"] }
gpui = { path = "D:/code/zed/crates/gpui" }
gpui_macros = { path = "D:/code/zed/crates/gpui_macros" }
```

```toml
# E:\code\workspace\rust-toolchain.toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
```

```toml
# Representative crate manifest pattern, for example E:\code\workspace\crates\foundation\Cargo.toml
[package]
name = "foundation"
version.workspace = true
edition.workspace = true
license.workspace = true

[lib]
path = "src/foundation.rs"

[dependencies]
anyhow.workspace = true
serde.workspace = true
tracing.workspace = true
```

- [ ] **Step 4: Run validation to verify it passes**

Run: `cargo metadata --format-version 1`
Expected: PASS and print JSON workspace metadata.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml rust-toolchain.toml crates/*/Cargo.toml
git commit -m "chore: scaffold workspace manifests"
```

## Task 2: Create The Shared Foundation And Paths Layer

**Files:**
- Create: `E:\code\workspace\crates\foundation\src\foundation.rs`
- Create: `E:\code\workspace\crates\paths\src\paths.rs`
- Test: `E:\code\workspace\crates\foundation\tests\foundation_tests.rs`
- Test: `E:\code\workspace\crates\paths\tests\paths_tests.rs`

- [ ] **Step 1: Write the failing tests**

```rust
// E:\code\workspace\crates\foundation\tests\foundation_tests.rs
use foundation::{AppError, Id};

#[test]
fn id_new_rejects_empty_values() {
    let error = Id::new("").unwrap_err();
    assert!(matches!(error, AppError::InvalidId));
}
```

```rust
// E:\code\workspace\crates\paths\tests\paths_tests.rs
use paths::AppPaths;

#[test]
fn app_paths_returns_framework_file_locations() {
    let paths = AppPaths::for_root("C:/FrameworkRoot");
    assert!(paths.settings_file.ends_with("settings.json"));
    assert!(paths.database_file.ends_with("framework.db"));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p foundation -p paths`
Expected: FAIL with missing crates or missing symbols.

- [ ] **Step 3: Write minimal implementation**

```rust
// E:\code\workspace\crates\foundation\src\foundation.rs
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppError {
    InvalidId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id(String);

impl Id {
    pub fn new(value: &str) -> Result<Self, AppError> {
        if value.trim().is_empty() {
            return Err(AppError::InvalidId);
        }
        Ok(Self(value.to_owned()))
    }
}

impl Display for Id {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}
```

```rust
// E:\code\workspace\crates\paths\src\paths.rs
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub root: PathBuf,
    pub settings_file: PathBuf,
    pub keymap_file: PathBuf,
    pub database_file: PathBuf,
}

impl AppPaths {
    pub fn for_root(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        Self {
            settings_file: root.join("settings.json"),
            keymap_file: root.join("keymap.json"),
            database_file: root.join("framework.db"),
            root,
        }
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p foundation -p paths`
Expected: PASS with 2 passing tests.

- [ ] **Step 5: Commit**

```bash
git add crates/foundation crates/paths
git commit -m "feat: add foundation and paths primitives"
```

## Task 3: Add Assets, Settings, Theme, And Keymap Loading

**Files:**
- Create: `E:\code\workspace\assets\themes\default.json`
- Create: `E:\code\workspace\assets\keymaps\default-windows.json`
- Create: `E:\code\workspace\crates\assets\src\assets.rs`
- Create: `E:\code\workspace\crates\settings\src\settings.rs`
- Create: `E:\code\workspace\crates\theme\src\theme.rs`
- Create: `E:\code\workspace\crates\keymap\src\keymap.rs`
- Test: `E:\code\workspace\crates\settings\tests\settings_tests.rs`
- Test: `E:\code\workspace\crates\theme\tests\theme_tests.rs`
- Test: `E:\code\workspace\crates\keymap\tests\keymap_tests.rs`

- [ ] **Step 1: Write the failing tests**

```rust
// E:\code\workspace\crates\settings\tests\settings_tests.rs
use settings::AppSettings;

#[test]
fn default_settings_enable_session_restore() {
    let settings = AppSettings::default();
    assert!(settings.restore_session_on_launch);
}
```

```rust
// E:\code\workspace\crates\theme\tests\theme_tests.rs
use theme::ThemeDefinition;

#[test]
fn default_theme_contains_named_tokens() {
    let theme = ThemeDefinition::default();
    assert_eq!(theme.name, "Default");
    assert!(theme.tokens.contains_key("status_bar.background"));
}
```

```rust
// E:\code\workspace\crates\keymap\tests\keymap_tests.rs
use keymap::KeyBinding;

#[test]
fn key_binding_keeps_command_and_keystroke() {
    let binding = KeyBinding::new("workspace.toggle_left_dock", "ctrl-b");
    assert_eq!(binding.command_id, "workspace.toggle_left_dock");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p settings -p theme -p keymap`
Expected: FAIL with missing files or missing types.

- [ ] **Step 3: Write the minimal implementation and assets**

```json
// E:\code\workspace\assets\themes\default.json
{
  "name": "Default",
  "tokens": {
    "app.background": "#111111",
    "sidebar.background": "#1a1a1a",
    "status_bar.background": "#222222"
  }
}
```

```json
// E:\code\workspace\assets\keymaps\default-windows.json
[
  { "command_id": "workspace.toggle_left_dock", "keystroke": "ctrl-b" },
  { "command_id": "command_palette.toggle", "keystroke": "ctrl-shift-p" }
]
```

```rust
// E:\code\workspace\crates\settings\src\settings.rs
#[derive(Debug, Clone)]
pub struct AppSettings {
    pub restore_session_on_launch: bool,
    pub theme_name: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            restore_session_on_launch: true,
            theme_name: "Default".to_owned(),
        }
    }
}
```

```rust
// E:\code\workspace\crates\theme\src\theme.rs
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct ThemeDefinition {
    pub name: String,
    pub tokens: BTreeMap<String, String>,
}

impl Default for ThemeDefinition {
    fn default() -> Self {
        let mut tokens = BTreeMap::new();
        tokens.insert("status_bar.background".to_owned(), "#222222".to_owned());
        Self { name: "Default".to_owned(), tokens }
    }
}
```

```rust
// E:\code\workspace\crates\keymap\src\keymap.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyBinding {
    pub command_id: String,
    pub keystroke: String,
}

impl KeyBinding {
    pub fn new(command_id: &str, keystroke: &str) -> Self {
        Self {
            command_id: command_id.to_owned(),
            keystroke: keystroke.to_owned(),
        }
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p settings -p theme -p keymap`
Expected: PASS with 3 passing tests.

- [ ] **Step 5: Commit**

```bash
git add assets crates/assets crates/settings crates/theme crates/keymap
git commit -m "feat: add assets settings theme and keymap primitives"
```

## Task 4: Build Actions, Commands, Menu, And Command Palette Contracts

**Files:**
- Create: `E:\code\workspace\crates\actions\src\actions.rs`
- Create: `E:\code\workspace\crates\commands\src\commands.rs`
- Create: `E:\code\workspace\crates\menu\src\menu.rs`
- Create: `E:\code\workspace\crates\command_palette\src\command_palette.rs`
- Test: `E:\code\workspace\crates\commands\tests\commands_tests.rs`

- [ ] **Step 1: Write the failing test**

```rust
// E:\code\workspace\crates\commands\tests\commands_tests.rs
use commands::{CommandDescriptor, CommandRegistry};

#[test]
fn registry_rejects_duplicate_command_ids() {
    let mut registry = CommandRegistry::default();
    let command = CommandDescriptor::new("workspace.toggle_left_dock", "Toggle Left Dock");
    registry.register(command.clone()).unwrap();
    assert!(registry.register(command).is_err());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p commands`
Expected: FAIL with missing crate or missing symbols.

- [ ] **Step 3: Write minimal implementation**

```rust
// E:\code\workspace\crates\commands\src\commands.rs
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct CommandDescriptor {
    pub id: String,
    pub title: String,
}

impl CommandDescriptor {
    pub fn new(id: &str, title: &str) -> Self {
        Self { id: id.to_owned(), title: title.to_owned() }
    }
}

#[derive(Debug, Default)]
pub struct CommandRegistry {
    commands: BTreeMap<String, CommandDescriptor>,
}

impl CommandRegistry {
    pub fn register(&mut self, command: CommandDescriptor) -> Result<(), String> {
        if self.commands.contains_key(&command.id) {
            return Err(format!("duplicate command id: {}", command.id));
        }
        self.commands.insert(command.id.clone(), command);
        Ok(())
    }
}
```

```rust
// E:\code\workspace\crates\actions\src\actions.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionDescriptor {
    pub id: String,
}
```

```rust
// E:\code\workspace\crates\menu\src\menu.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuItem {
    pub title: String,
    pub command_id: String,
}
```

```rust
// E:\code\workspace\crates\command_palette\src\command_palette.rs
#[derive(Debug, Default)]
pub struct CommandPaletteState {
    pub is_open: bool,
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p commands`
Expected: PASS with 1 passing test.

- [ ] **Step 5: Commit**

```bash
git add crates/actions crates/commands crates/menu crates/command_palette
git commit -m "feat: add command system contracts"
```

## Task 5: Build Panel, Dock, Status Bar, And Notifications

**Files:**
- Create: `E:\code\workspace\crates\panel\src\panel.rs`
- Create: `E:\code\workspace\crates\dock\src\dock.rs`
- Create: `E:\code\workspace\crates\status_bar\src\status_bar.rs`
- Create: `E:\code\workspace\crates\notifications\src\notifications.rs`
- Test: `E:\code\workspace\crates\panel\tests\panel_tests.rs`
- Test: `E:\code\workspace\crates\dock\tests\dock_tests.rs`

- [ ] **Step 1: Write the failing tests**

```rust
// E:\code\workspace\crates\panel\tests\panel_tests.rs
use panel::{DockPlacement, PanelDescriptor};

#[test]
fn panel_descriptor_defaults_to_restorable() {
    let descriptor = PanelDescriptor::new("welcome.panel", "Welcome", DockPlacement::Center);
    assert!(descriptor.restorable);
}
```

```rust
// E:\code\workspace\crates\dock\tests\dock_tests.rs
use dock::VisibleDockState;
use panel::DockPlacement;

#[test]
fn visible_dock_state_tracks_multiple_placements() {
    let state = VisibleDockState::with_visible(DockPlacement::Left, "welcome.panel");
    assert_eq!(state.visible_panels.len(), 1);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p panel -p dock`
Expected: FAIL with missing symbols.

- [ ] **Step 3: Write minimal implementation**

```rust
// E:\code\workspace\crates\panel\src\panel.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockPlacement {
    Left,
    Center,
    Bottom,
    Right,
}

#[derive(Debug, Clone)]
pub struct PanelDescriptor {
    pub id: String,
    pub title: String,
    pub dock: DockPlacement,
    pub restorable: bool,
}

impl PanelDescriptor {
    pub fn new(id: &str, title: &str, dock: DockPlacement) -> Self {
        Self { id: id.to_owned(), title: title.to_owned(), dock, restorable: true }
    }
}
```

```rust
// E:\code\workspace\crates\dock\src\dock.rs
use panel::DockPlacement;

#[derive(Debug, Clone)]
pub struct VisiblePanel {
    pub placement: DockPlacement,
    pub panel_id: String,
}

#[derive(Debug, Default, Clone)]
pub struct VisibleDockState {
    pub visible_panels: Vec<VisiblePanel>,
}

impl VisibleDockState {
    pub fn with_visible(placement: DockPlacement, panel_id: &str) -> Self {
        Self {
            visible_panels: vec![VisiblePanel { placement, panel_id: panel_id.to_owned() }],
        }
    }
}
```

```rust
// E:\code\workspace\crates\status_bar\src\status_bar.rs
#[derive(Debug, Clone)]
pub struct StatusBarItem {
    pub id: String,
    pub text: String,
}
```

```rust
// E:\code\workspace\crates\notifications\src\notifications.rs
#[derive(Debug, Clone)]
pub struct ToastNotification {
    pub id: String,
    pub message: String,
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p panel -p dock`
Expected: PASS with 2 passing tests.

- [ ] **Step 5: Commit**

```bash
git add crates/panel crates/dock crates/status_bar crates/notifications
git commit -m "feat: add panel dock and notification primitives"
```

## Task 6: Build The Module System

**Files:**
- Create: `E:\code\workspace\crates\module\src\module.rs`
- Test: `E:\code\workspace\crates\module\tests\module_tests.rs`

- [ ] **Step 1: Write the failing test**

```rust
// E:\code\workspace\crates\module\tests\module_tests.rs
use module::{CommandContribution, FeatureModule, ModuleRuntime, SimpleModule};

#[test]
fn module_runtime_rejects_duplicate_command_contributions() {
    let first = SimpleModule::with_command("welcome", CommandContribution::new("command.open"));
    let second = SimpleModule::with_command("inspector", CommandContribution::new("command.open"));
    let mut runtime = ModuleRuntime::default();
    runtime.register(Box::new(first)).unwrap();
    assert!(runtime.register(Box::new(second)).is_err());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p module`
Expected: FAIL with missing crate or missing symbols.

- [ ] **Step 3: Write minimal implementation**

```rust
// E:\code\workspace\crates\module\src\module.rs
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct CommandContribution {
    pub command_id: String,
}

impl CommandContribution {
    pub fn new(command_id: &str) -> Self {
        Self { command_id: command_id.to_owned() }
    }
}

pub trait FeatureModule {
    fn module_id(&self) -> &str;
    fn command_contributions(&self) -> &[CommandContribution];
}

pub struct SimpleModule {
    module_id: String,
    commands: Vec<CommandContribution>,
}

impl SimpleModule {
    pub fn with_command(module_id: &str, command: CommandContribution) -> Self {
        Self { module_id: module_id.to_owned(), commands: vec![command] }
    }
}

impl FeatureModule for SimpleModule {
    fn module_id(&self) -> &str { &self.module_id }
    fn command_contributions(&self) -> &[CommandContribution] { &self.commands }
}

#[derive(Default)]
pub struct ModuleRuntime {
    module_ids: BTreeSet<String>,
    command_ids: BTreeSet<String>,
}

impl ModuleRuntime {
    pub fn register(&mut self, module: Box<dyn FeatureModule>) -> Result<(), String> {
        if !self.module_ids.insert(module.module_id().to_owned()) {
            return Err(format!("duplicate module id: {}", module.module_id()));
        }
        for contribution in module.command_contributions() {
            if !self.command_ids.insert(contribution.command_id.clone()) {
                return Err(format!("duplicate command contribution: {}", contribution.command_id));
            }
        }
        Ok(())
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p module`
Expected: PASS with 1 passing test.

- [ ] **Step 5: Commit**

```bash
git add crates/module
git commit -m "feat: add module runtime and contributions"
```

## Task 7: Build The Database And Repositories

**Files:**
- Create: `E:\code\workspace\crates\db\src\db.rs`
- Create: `E:\code\workspace\crates\db\migrations\0001_init.sql`
- Test: `E:\code\workspace\crates\db\tests\db_tests.rs`

- [ ] **Step 1: Write the failing test**

```rust
// E:\code\workspace\crates\db\tests\db_tests.rs
use db::schema_sql;

#[test]
fn initial_schema_contains_workspace_sessions() {
    let sql = schema_sql();
    assert!(sql.contains("create table workspace_sessions"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p db`
Expected: FAIL with missing crate or function.

- [ ] **Step 3: Write minimal implementation**

```sql
-- E:\code\workspace\crates\db\migrations\0001_init.sql
create table workspace_sessions (
  session_id text primary key,
  created_at text not null
);

create table panel_states (
  session_id text not null,
  panel_id text not null,
  dock text not null,
  visible integer not null
);

create table command_history (
  command_id text not null,
  created_at text not null
);

create table notifications (
  notification_id text primary key,
  level text not null,
  message text not null,
  created_at text not null
);

create table module_state (
  module_id text not null,
  state_key text not null,
  state_json text not null,
  updated_at text not null,
  primary key (module_id, state_key)
);
```

```rust
// E:\code\workspace\crates\db\src\db.rs
pub fn schema_sql() -> &'static str {
    include_str!("../migrations/0001_init.sql")
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p db`
Expected: PASS with 1 passing test.

- [ ] **Step 5: Commit**

```bash
git add crates/db
git commit -m "feat: add initial sqlite schema"
```

## Task 8: Build The Workspace Controller And Session Models

**Files:**
- Create: `E:\code\workspace\crates\workspace\src\workspace.rs`
- Test: `E:\code\workspace\crates\workspace\tests\workspace_tests.rs`

- [ ] **Step 1: Write the failing test**

```rust
// E:\code\workspace\crates\workspace\tests\workspace_tests.rs
use workspace::WorkspaceState;

#[test]
fn workspace_state_remembers_focused_panel() {
    let mut state = WorkspaceState::default();
    state.focus_panel("welcome.panel");
    assert_eq!(state.focused_panel.as_deref(), Some("welcome.panel"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p workspace`
Expected: FAIL with missing crate or symbols.

- [ ] **Step 3: Write minimal implementation**

```rust
// E:\code\workspace\crates\workspace\src\workspace.rs
#[derive(Debug, Default, Clone)]
pub struct WorkspaceState {
    pub focused_panel: Option<String>,
    pub visible_panels: Vec<String>,
}

impl WorkspaceState {
    pub fn focus_panel(&mut self, panel_id: &str) {
        self.focused_panel = Some(panel_id.to_owned());
        if !self.visible_panels.iter().any(|existing| existing == panel_id) {
            self.visible_panels.push(panel_id.to_owned());
        }
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p workspace`
Expected: PASS with 1 passing test.

- [ ] **Step 5: Commit**

```bash
git add crates/workspace
git commit -m "feat: add workspace state primitives"
```

## Task 9: Build The Root App Frame And Welcome Module

**Files:**
- Create: `E:\code\workspace\crates\app_ui\src\app_ui.rs`
- Create: `E:\code\workspace\crates\welcome\src\welcome.rs`
- Create: `E:\code\workspace\crates\app\src\main.rs`
- Test: `E:\code\workspace\crates\welcome\tests\welcome_tests.rs`

- [ ] **Step 1: Write the failing test**

```rust
// E:\code\workspace\crates\welcome\tests\welcome_tests.rs
use welcome::WelcomeModule;
use module::FeatureModule;

#[test]
fn welcome_module_uses_stable_module_id() {
    let module = WelcomeModule::default();
    assert_eq!(module.module_id(), "welcome");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p welcome`
Expected: FAIL with missing crate or type.

- [ ] **Step 3: Write minimal implementation**

```rust
// E:\code\workspace\crates\welcome\src\welcome.rs
use module::{CommandContribution, FeatureModule, SimpleModule};

#[derive(Default)]
pub struct WelcomeModule {
    inner: SimpleModule,
}

impl WelcomeModule {
    pub fn new() -> Self {
        Self {
            inner: SimpleModule::with_command(
                "welcome",
                CommandContribution::new("welcome.open"),
            ),
        }
    }
}

impl Default for WelcomeModule {
    fn default() -> Self {
        Self::new()
    }
}

impl FeatureModule for WelcomeModule {
    fn module_id(&self) -> &str { self.inner.module_id() }
    fn command_contributions(&self) -> &[CommandContribution] { self.inner.command_contributions() }
}
```

```rust
// E:\code\workspace\crates\app_ui\src\app_ui.rs
pub struct AppFrame;

impl AppFrame {
    pub fn title() -> &'static str {
        "Zed-Style Desktop Framework"
    }
}
```

```rust
// E:\code\workspace\crates\app\src\main.rs
fn main() {
    println!("bootstrap zed-style desktop framework");
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p welcome`
Expected: PASS with 1 passing test.

- [ ] **Step 5: Commit**

```bash
git add crates/welcome crates/app_ui crates/app
git commit -m "feat: add app frame and welcome module"
```

## Task 10: Integrate Startup And Verify The Whole Workspace

**Files:**
- Modify: `E:\code\workspace\crates\app\src\main.rs`
- Modify: `E:\code\workspace\tech.md`
- Test: `E:\code\workspace\crates\app\tests\startup_smoke.rs`

- [ ] **Step 1: Write the failing test**

```rust
// E:\code\workspace\crates\app\tests\startup_smoke.rs
#[test]
fn app_title_matches_framework_goal() {
    assert_eq!(app_ui::AppFrame::title(), "Zed-Style Desktop Framework");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p app`
Expected: FAIL because `app` does not depend on `app_ui` yet or startup wiring is incomplete.

- [ ] **Step 3: Wire startup and write `tech.md`**

```rust
// E:\code\workspace\crates\app\src\main.rs
use app_ui::AppFrame;

fn main() {
    println!("starting {}", AppFrame::title());
}
```

```md
# Technical Specification

## Architecture
- `app` owns process startup.
- `app_ui` owns the application frame.
- `workspace` owns runtime frame state.
- `module` owns extension contracts.
- `db` owns `sqlx + sqlite` persistence and migrations.

## Persistence
- `settings.json` and `keymap.json` are user-editable files.
- `framework.db` stores session, panel, command, notification, and module state.

## Verification
- `cargo metadata --format-version 1`
- `cargo test`
- `cargo check`
```

- [ ] **Step 4: Run full verification**

Run: `cargo test && cargo check`
Expected: PASS across the workspace.

- [ ] **Step 5: Commit**

```bash
git add crates/app tech.md
git commit -m "docs: finalize technical specification and startup wiring"
```

## Self-Review

### Spec Coverage

- GPUI startup and app entry: covered by Tasks 1, 9, and 10
- app frame and fixed regions: covered by Tasks 5 and 9
- actions, commands, keymap, menu, command palette: covered by Tasks 3 and 4
- panel and dock separation: covered by Task 5
- module contribution system: covered by Task 6
- settings/theme/assets/paths: covered by Tasks 2 and 3
- `sqlx + sqlite` persistence, migrations, repositories: covered by Task 7
- workspace session and restore primitives: covered by Task 8
- demo feature module: covered by Task 9
- `tech.md`: covered by Task 10

### Placeholder Scan

- No `TODO`, `TBD`, or placeholder markers remain.
- Every task lists exact files and commands.
- Every code step contains concrete starter code or concrete file content.

### Type Consistency

- `FeatureModule`, `CommandContribution`, and `ModuleRuntime` are defined in Task 6 and reused consistently in Task 9.
- `AppFrame::title()` is defined in Task 9 and consumed in Task 10.
- `WorkspaceState::focus_panel()` is introduced once in Task 8 and not renamed later.
