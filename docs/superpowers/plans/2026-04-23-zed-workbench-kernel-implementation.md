# Zed Workbench Kernel Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the fake Zed-like shell with a real Zed-style workbench kernel that supports typed actions, singleton panel lifecycle, dock runtime state, restore-safe persistence, and one real `welcome` panel.

**Architecture:** Keep this repository's own `workspace`, `dock`, `panel`, `actions`, and `app_ui` crates, but reshape them around a real runtime model. Treat all shell surfaces as action dispatchers into `workspace`, make explorer a built-in left-dock panel type, and validate the full lifecycle with a singleton `welcome` panel before adding more business modules.

**Tech Stack:** Rust 2024, `gpui`, `gpui-component`, `sqlx/sqlite`, workspace-local crates (`panel`, `dock`, `workspace`, `actions`, `commands`, `module`, `welcome`, `app_ui`, `ui`, `db`)

---

## File Structure

### Foundation Runtime

- `crates/panel/src/panel.rs`
  - Define panel type identity, instance identity, multiplicity, close behavior, and serialized panel records.
- `crates/dock/src/dock.rs`
  - Hold left/center/right/bottom dock containers, active tab per dock, and visibility state.
- `crates/actions/src/actions.rs`
  - Define typed shell actions and action envelopes shared by menu, keymap, and command palette.
- `crates/commands/src/commands.rs`
  - Map commands to typed actions instead of only carrying titles.
- `crates/workspace/src/workspace.rs`
  - Implement panel registry, singleton instance store, action execution, dock mutation, and restore-safe serialization.
- `crates/module/src/module.rs`
  - Upgrade module registration to register panel types and actions, not only panel metadata and command ids.
- `crates/db/migrations/0002_workspace_shell_session.sql`
  - Add schema-versioned shell session persistence fields for singleton instance restore.
- `crates/db/src/db.rs`
  - Expose migration SQL loading for the second schema.

### Shell Composition

- `crates/app/src/main.rs`
  - Bootstrap the runtime, install typed action handlers, and configure Zed-style window options.
- `crates/app_ui/src/app_ui.rs`
  - Render title bar host, left/right/bottom/center dock hosts, and contribution-driven status bar.
- `crates/ui/src/ui.rs`
  - Remove fake shell content and render runtime-backed shell hosts only.

### Built-In Validation Module

- `crates/welcome/src/welcome.rs`
  - Register `welcome` as a singleton panel type and map its command/action contributions.
- `crates/welcome/src/welcome_panel.rs`
  - Render the real `welcome` panel entity and serialize its local panel state.

### Tests

- `crates/panel/tests/panel_identity_tests.rs`
- `crates/dock/tests/dock_runtime_tests.rs`
- `crates/actions/tests/action_contract_tests.rs`
- `crates/commands/tests/command_action_mapping_tests.rs`
- `crates/workspace/tests/workspace_runtime_tests.rs`
- `crates/welcome/tests/welcome_panel_tests.rs`
- `crates/app/tests/bootstrap_runtime_tests.rs`
- `crates/db/tests/db_tests.rs`

## Task 1: Define Panel Identity and Singleton Lifetime Rules

**Files:**
- Modify: `crates/panel/src/panel.rs`
- Test: `crates/panel/tests/panel_identity_tests.rs`

- [ ] **Step 1: Write the failing panel identity tests**

```rust
use panel::{
    DockPlacement, PanelCloseBehavior, PanelDescriptor, PanelInstanceId, PanelInstanceKey,
    PanelMultiplicity, PanelTypeId,
};

#[test]
fn singleton_panel_descriptor_uses_type_id_as_default_instance_key() {
    let descriptor = PanelDescriptor::singleton(
        PanelTypeId::try_new("welcome.panel").expect("panel type id should parse"),
        "Welcome",
        DockPlacement::Center,
    );

    assert_eq!(descriptor.multiplicity(), PanelMultiplicity::Singleton);
    assert_eq!(descriptor.close_behavior(), PanelCloseBehavior::Hide);
    assert_eq!(
        descriptor.default_instance_key(),
        &PanelInstanceKey::from_static("welcome.panel"),
    );
}

#[test]
fn panel_instance_id_is_distinct_from_panel_type_id() {
    let panel_type_id = PanelTypeId::try_new("explorer.panel").expect("panel type id should parse");
    let panel_instance_id = PanelInstanceId::new("explorer.panel#1");

    assert_eq!(panel_type_id.as_str(), "explorer.panel");
    assert_eq!(panel_instance_id.as_str(), "explorer.panel#1");
    assert_ne!(panel_type_id.as_str(), panel_instance_id.as_str());
}
```

- [ ] **Step 2: Run the panel test to verify it fails**

Run: `cargo test -p panel singleton_panel_descriptor_uses_type_id_as_default_instance_key -- --nocapture`

Expected: FAIL with unresolved imports such as `PanelTypeId`, `PanelMultiplicity`, or `PanelCloseBehavior`.

- [ ] **Step 3: Write the minimal panel identity implementation**

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct PanelTypeId(String);

impl PanelTypeId {
    pub fn try_new(value: impl Into<String>) -> Result<Self, PanelDescriptorError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(PanelDescriptorError::EmptyId);
        }
        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct PanelInstanceId(String);

impl PanelInstanceId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct PanelInstanceKey(String);

impl PanelInstanceKey {
    pub fn from_static(value: &'static str) -> Self {
        Self(value.to_owned())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelMultiplicity {
    Singleton,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelCloseBehavior {
    Hide,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelDescriptor {
    id: PanelTypeId,
    title: String,
    dock: DockPlacement,
    multiplicity: PanelMultiplicity,
    close_behavior: PanelCloseBehavior,
    default_instance_key: PanelInstanceKey,
    restorable: bool,
}

impl PanelDescriptor {
    pub fn singleton(id: PanelTypeId, title: impl Into<String>, dock: DockPlacement) -> Self {
        let default_instance_key = PanelInstanceKey(id.as_str().to_owned());
        Self {
            id,
            title: title.into(),
            dock,
            multiplicity: PanelMultiplicity::Singleton,
            close_behavior: PanelCloseBehavior::Hide,
            default_instance_key,
            restorable: true,
        }
    }

    pub fn multiplicity(&self) -> PanelMultiplicity {
        self.multiplicity
    }

    pub fn close_behavior(&self) -> PanelCloseBehavior {
        self.close_behavior
    }

    pub fn default_instance_key(&self) -> &PanelInstanceKey {
        &self.default_instance_key
    }
}
```

- [ ] **Step 4: Run the panel crate tests to verify they pass**

Run: `cargo test -p panel -- --nocapture`

Expected: PASS with the new identity tests and the existing panel tests.

- [ ] **Step 5: Commit**

```bash
git add crates/panel/src/panel.rs crates/panel/tests/panel_identity_tests.rs
git commit -m "feat: add panel identity and singleton rules"
```

## Task 2: Replace VisibleDockState With Real Dock Runtime State

**Files:**
- Modify: `crates/dock/src/dock.rs`
- Test: `crates/dock/tests/dock_runtime_tests.rs`

- [ ] **Step 1: Write the failing dock runtime tests**

```rust
use dock::{DockLayoutState, DockPlacement};
use panel::PanelInstanceKey;

#[test]
fn opening_a_panel_sets_the_active_tab_for_its_dock() {
    let mut layout = DockLayoutState::default();
    let key = PanelInstanceKey::from_static("welcome.panel");

    layout.show_panel(DockPlacement::Center, key.clone());

    assert_eq!(layout.active_panel(DockPlacement::Center), Some(&key));
    assert!(layout.is_visible(DockPlacement::Center));
}

#[test]
fn moving_a_panel_removes_it_from_the_old_dock_and_activates_it_in_the_new_dock() {
    let mut layout = DockLayoutState::default();
    let key = PanelInstanceKey::from_static("welcome.panel");

    layout.show_panel(DockPlacement::Center, key.clone());
    layout.move_panel(&key, DockPlacement::Right);

    assert_eq!(layout.active_panel(DockPlacement::Center), None);
    assert_eq!(layout.active_panel(DockPlacement::Right), Some(&key));
}
```

- [ ] **Step 2: Run the dock test to verify it fails**

Run: `cargo test -p dock opening_a_panel_sets_the_active_tab_for_its_dock -- --nocapture`

Expected: FAIL because `DockLayoutState` and runtime dock helpers do not exist.

- [ ] **Step 3: Write the minimal dock runtime implementation**

```rust
use std::collections::BTreeMap;
use panel::PanelInstanceKey;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DockContainerState {
    pub tabs: Vec<PanelInstanceKey>,
    pub active: Option<PanelInstanceKey>,
    pub visible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DockLayoutState {
    containers: BTreeMap<DockPlacement, DockContainerState>,
}

impl DockLayoutState {
    pub fn show_panel(&mut self, placement: DockPlacement, panel_key: PanelInstanceKey) {
        let container = self.containers.entry(placement).or_default();
        if !container.tabs.iter().any(|existing| existing == &panel_key) {
            container.tabs.push(panel_key.clone());
        }
        container.active = Some(panel_key);
        container.visible = true;
    }

    pub fn move_panel(&mut self, panel_key: &PanelInstanceKey, new_placement: DockPlacement) {
        for container in self.containers.values_mut() {
            container.tabs.retain(|existing| existing != panel_key);
            if container.active.as_ref() == Some(panel_key) {
                container.active = container.tabs.last().cloned();
            }
            container.visible = !container.tabs.is_empty();
        }
        self.show_panel(new_placement, panel_key.clone());
    }

    pub fn active_panel(&self, placement: DockPlacement) -> Option<&PanelInstanceKey> {
        self.containers.get(&placement).and_then(|container| container.active.as_ref())
    }

    pub fn is_visible(&self, placement: DockPlacement) -> bool {
        self.containers
            .get(&placement)
            .map(|container| container.visible)
            .unwrap_or(false)
    }
}
```

- [ ] **Step 4: Run the dock crate tests to verify they pass**

Run: `cargo test -p dock -- --nocapture`

Expected: PASS with the new dock runtime tests and the existing dock tests.

- [ ] **Step 5: Commit**

```bash
git add crates/dock/src/dock.rs crates/dock/tests/dock_runtime_tests.rs
git commit -m "feat: add dock runtime layout state"
```

## Task 3: Introduce a Typed Action Contract and Command Mapping

**Files:**
- Modify: `crates/actions/src/actions.rs`
- Modify: `crates/commands/src/commands.rs`
- Test: `crates/actions/tests/action_contract_tests.rs`
- Test: `crates/commands/tests/command_action_mapping_tests.rs`

- [ ] **Step 1: Write the failing action and command mapping tests**

```rust
use actions::{ActionEnvelope, PanelAction};
use commands::CommandDescriptor;

#[test]
fn panel_action_envelope_serializes_a_welcome_toggle() {
    let envelope = ActionEnvelope::panel(PanelAction::Toggle {
        panel_type_id: "welcome.panel".to_owned(),
    });

    let json = serde_json::to_string(&envelope).expect("action envelope should serialize");
    assert_eq!(json, "{\"kind\":\"panel\",\"action\":\"toggle\",\"panel_type_id\":\"welcome.panel\"}");
}

#[test]
fn command_descriptor_carries_a_typed_action() {
    let descriptor = CommandDescriptor::try_new(
        "welcome.open",
        "Open Welcome",
        ActionEnvelope::panel(PanelAction::Open {
            panel_type_id: "welcome.panel".to_owned(),
        }),
    )
    .expect("command descriptor should be valid");

    assert_eq!(descriptor.action().kind(), "panel");
}
```

- [ ] **Step 2: Run the action test to verify it fails**

Run: `cargo test -p actions panel_action_envelope_serializes_a_welcome_toggle -- --nocapture`

Expected: FAIL because `ActionEnvelope` and `PanelAction` do not exist.

- [ ] **Step 3: Write the minimal typed action and command mapping implementation**

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ActionEnvelope {
    #[serde(rename = "panel")]
    Panel(PanelActionEnvelope),
}

impl ActionEnvelope {
    pub fn panel(action: PanelAction) -> Self {
        Self::Panel(PanelActionEnvelope::from(action))
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Self::Panel(_) => "panel",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PanelAction {
    Open { panel_type_id: String },
    Toggle { panel_type_id: String },
    Focus { panel_type_id: String },
    Close { panel_type_id: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PanelActionEnvelope {
    pub action: String,
    pub panel_type_id: String,
}

impl From<PanelAction> for PanelActionEnvelope {
    fn from(value: PanelAction) -> Self {
        match value {
            PanelAction::Open { panel_type_id } => Self { action: "open".to_owned(), panel_type_id },
            PanelAction::Toggle { panel_type_id } => Self { action: "toggle".to_owned(), panel_type_id },
            PanelAction::Focus { panel_type_id } => Self { action: "focus".to_owned(), panel_type_id },
            PanelAction::Close { panel_type_id } => Self { action: "close".to_owned(), panel_type_id },
        }
    }
}
```

```rust
use actions::ActionEnvelope;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandDescriptor {
    id: CommandId,
    title: CommandTitle,
    action: ActionEnvelope,
}

impl CommandDescriptor {
    pub fn try_new(
        id: impl Into<String>,
        title: impl Into<String>,
        action: ActionEnvelope,
    ) -> Result<Self, CommandDescriptorError> {
        Ok(Self {
            id: CommandId::try_new(id)?,
            title: CommandTitle::try_new(title)?,
            action,
        })
    }

    pub fn action(&self) -> &ActionEnvelope {
        &self.action
    }
}
```

- [ ] **Step 4: Run the actions and commands tests to verify they pass**

Run: `cargo test -p actions -p commands -- --nocapture`

Expected: PASS with the new action contract and command mapping tests.

- [ ] **Step 5: Commit**

```bash
git add crates/actions/src/actions.rs crates/actions/tests/action_contract_tests.rs crates/commands/src/commands.rs crates/commands/tests/command_action_mapping_tests.rs
git commit -m "feat: add typed action envelopes for shell commands"
```

## Task 4: Rebuild Workspace Runtime Around Singleton Panels and Versioned Restore

**Files:**
- Modify: `crates/workspace/src/workspace.rs`
- Test: `crates/workspace/tests/workspace_runtime_tests.rs`

- [ ] **Step 1: Write the failing workspace runtime tests**

```rust
use actions::{ActionEnvelope, PanelAction};
use panel::{DockPlacement, PanelDescriptor, PanelTypeId};
use workspace::{SerializedWorkspaceSession, WorkspaceController};

#[test]
fn toggle_panel_opens_then_hides_the_same_singleton_instance() {
    let mut workspace = WorkspaceController::new("session-1");
    workspace.register_panel(PanelDescriptor::singleton(
        PanelTypeId::try_new("welcome.panel").expect("panel type id should parse"),
        "Welcome",
        DockPlacement::Center,
    ));

    workspace.dispatch(ActionEnvelope::panel(PanelAction::Toggle {
        panel_type_id: "welcome.panel".to_owned(),
    }));
    assert!(workspace.state().is_panel_visible("welcome.panel"));

    workspace.dispatch(ActionEnvelope::panel(PanelAction::Toggle {
        panel_type_id: "welcome.panel".to_owned(),
    }));
    assert!(!workspace.state().is_panel_visible("welcome.panel"));
}

#[test]
fn restore_skips_unknown_panels_and_keeps_startup_alive() {
    let mut workspace = WorkspaceController::new("session-1");
    let session = SerializedWorkspaceSession {
        schema_version: 2,
        panels: vec![workspace::SerializedPanelState {
            panel_type_id: "missing.panel".to_owned(),
            panel_instance_key: "missing.panel".to_owned(),
            placement: DockPlacement::Right,
            visible: true,
            focused: true,
            panel_state_json: Some("{}".to_owned()),
        }],
    };

    workspace.restore_session(&session);

    assert!(workspace.state().visible_panels().is_empty());
    assert!(workspace.session().restored);
}
```

- [ ] **Step 2: Run the workspace test to verify it fails**

Run: `cargo test -p workspace toggle_panel_opens_then_hides_the_same_singleton_instance -- --nocapture`

Expected: FAIL because `dispatch`, `SerializedWorkspaceSession`, and singleton lifecycle helpers do not exist.

- [ ] **Step 3: Write the minimal workspace runtime implementation**

```rust
use actions::{ActionEnvelope, PanelActionEnvelope};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SerializedPanelState {
    pub panel_type_id: String,
    pub panel_instance_key: String,
    pub placement: DockPlacement,
    pub visible: bool,
    pub focused: bool,
    pub panel_state_json: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct SerializedWorkspaceSession {
    pub schema_version: u32,
    pub panels: Vec<SerializedPanelState>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PanelRuntimeState {
    pub descriptor: PanelDescriptor,
    pub instance_id: PanelInstanceId,
    pub instance_key: PanelInstanceKey,
    pub visible: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorkspaceState {
    registered_panels: BTreeMap<String, PanelDescriptor>,
    live_panels: BTreeMap<String, PanelRuntimeState>,
    dock_layout: DockLayoutState,
    focused_panel: Option<String>,
    recent_commands: Vec<String>,
}

impl WorkspaceState {
    pub fn dispatch(&mut self, action: &ActionEnvelope) {
        match action {
            ActionEnvelope::Panel(PanelActionEnvelope { action, panel_type_id }) if action == "toggle" => {
                if self.is_panel_visible(panel_type_id) {
                    self.hide_panel(panel_type_id);
                } else {
                    self.open_panel(panel_type_id);
                }
            }
            ActionEnvelope::Panel(PanelActionEnvelope { action, panel_type_id }) if action == "open" => {
                self.open_panel(panel_type_id);
            }
            _ => {}
        }
    }

    pub fn open_panel(&mut self, panel_type_id: &str) {
        let descriptor = match self.registered_panels.get(panel_type_id).cloned() {
            Some(descriptor) => descriptor,
            None => return,
        };
        let runtime = self.live_panels.entry(panel_type_id.to_owned()).or_insert_with(|| PanelRuntimeState {
            instance_id: PanelInstanceId::new(format!("{panel_type_id}#singleton")),
            instance_key: descriptor.default_instance_key().clone(),
            descriptor: descriptor.clone(),
            visible: false,
        });
        runtime.visible = true;
        self.focused_panel = Some(panel_type_id.to_owned());
        self.dock_layout.show_panel(descriptor.dock, runtime.instance_key.clone());
    }

    pub fn hide_panel(&mut self, panel_type_id: &str) {
        if let Some(runtime) = self.live_panels.get_mut(panel_type_id) {
            runtime.visible = false;
        }
        if self.focused_panel.as_deref() == Some(panel_type_id) {
            self.focused_panel = None;
        }
    }
}
```

- [ ] **Step 4: Run the workspace crate tests to verify they pass**

Run: `cargo test -p workspace -- --nocapture`

Expected: PASS with lifecycle and degraded restore coverage.

- [ ] **Step 5: Commit**

```bash
git add crates/workspace/src/workspace.rs crates/workspace/tests/workspace_runtime_tests.rs
git commit -m "feat: add singleton workspace runtime lifecycle"
```

## Task 5: Upgrade Module Registration and Add the Real Welcome Panel

**Files:**
- Modify: `crates/module/src/module.rs`
- Modify: `crates/welcome/src/welcome.rs`
- Create: `crates/welcome/src/welcome_panel.rs`
- Test: `crates/welcome/tests/welcome_panel_tests.rs`
- Test: `crates/module/tests/module_tests.rs`

- [ ] **Step 1: Write the failing module and welcome tests**

```rust
use module::ModuleRuntime;
use welcome::WelcomeModule;

#[test]
fn welcome_module_registers_a_singleton_panel_type_and_open_action() {
    let mut runtime = ModuleRuntime::default();
    runtime.register(Box::new(WelcomeModule::default())).expect("welcome module should register");

    let registered = runtime.retained_modules().first().expect("welcome module should be retained");
    assert_eq!(registered.panels()[0].id().as_str(), "welcome.panel");
    assert_eq!(registered.commands()[0].action().kind(), "panel");
}
```

```rust
use welcome::WelcomePanelState;

#[test]
fn welcome_panel_state_round_trips_with_defaults() {
    let state = WelcomePanelState::default();
    let json = serde_json::to_string(&state).expect("welcome panel state should serialize");
    let restored: WelcomePanelState = serde_json::from_str(&json).expect("welcome panel state should deserialize");

    assert_eq!(state, restored);
}
```

- [ ] **Step 2: Run the welcome test to verify it fails**

Run: `cargo test -p welcome welcome_panel_state_round_trips_with_defaults -- --nocapture`

Expected: FAIL because `WelcomePanelState` and the real panel entity do not exist.

- [ ] **Step 3: Write the minimal module and welcome implementation**

```rust
use actions::{ActionEnvelope, PanelAction};

pub struct CommandContribution {
    command: CommandDescriptor,
}

impl CommandContribution {
    pub fn descriptor(&self) -> &CommandDescriptor {
        &self.command
    }
}

pub trait FeatureModule {
    fn module_id(&self) -> &str;
    fn command_contributions(&self) -> &[CommandContribution];
    fn panel_contributions(&self) -> &[PanelContribution];
}
```

```rust
use panel::{DockPlacement, PanelDescriptor, PanelTypeId};

#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WelcomePanelState {
    pub mode: String,
}

pub fn welcome_panel_descriptor() -> PanelDescriptor {
    PanelDescriptor::singleton(
        PanelTypeId::try_new("welcome.panel").expect("built-in welcome panel id should be valid"),
        "Welcome",
        DockPlacement::Center,
    )
}

fn welcome_command_contribution() -> CommandContribution {
    CommandContribution::new(
        CommandDescriptor::try_new(
            "welcome.open",
            "Open Welcome",
            ActionEnvelope::panel(PanelAction::Open {
                panel_type_id: "welcome.panel".to_owned(),
            }),
        )
        .expect("built-in welcome command should be valid"),
    )
}
```

- [ ] **Step 4: Run the module and welcome crate tests to verify they pass**

Run: `cargo test -p module -p welcome -- --nocapture`

Expected: PASS with action-carrying command contributions and serializable `WelcomePanelState`.

- [ ] **Step 5: Commit**

```bash
git add crates/module/src/module.rs crates/module/tests/module_tests.rs crates/welcome/src/welcome.rs crates/welcome/src/welcome_panel.rs crates/welcome/tests/welcome_panel_tests.rs
git commit -m "feat: register welcome as a real singleton panel"
```

## Task 6: Add Schema-Versioned Session Persistence and DB Coverage

**Files:**
- Create: `crates/db/migrations/0002_workspace_shell_session.sql`
- Modify: `crates/db/src/db.rs`
- Modify: `crates/db/tests/db_tests.rs`

- [ ] **Step 1: Write the failing DB migration test**

```rust
#[tokio::test]
async fn second_migration_tracks_schema_version_and_panel_instance_keys() {
    let mut connection = connect_and_migrate_all().await;

    sqlx::query(
        "insert into workspace_sessions (session_id, schema_version, created_at) values (?, ?, ?)",
    )
    .bind("session-2")
    .bind(2_i64)
    .bind("2026-04-23T00:00:00Z")
    .execute(&mut connection)
    .await
    .expect("workspace session with schema version should insert");

    sqlx::query(
        "insert into panel_states (session_id, panel_id, panel_instance_key, dock, visible, focused, panel_state_json) values (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("session-2")
    .bind("welcome.panel")
    .bind("welcome.panel")
    .bind("Center")
    .bind(1_i64)
    .bind(1_i64)
    .bind("{\"mode\":\"default\"}")
    .execute(&mut connection)
    .await
    .expect("panel state with instance key should insert");
}
```

- [ ] **Step 2: Run the DB test to verify it fails**

Run: `cargo test -p db second_migration_tracks_schema_version_and_panel_instance_keys -- --nocapture`

Expected: FAIL because the schema and migration loader do not include schema version or `panel_instance_key`.

- [ ] **Step 3: Write the minimal schema and loader changes**

```sql
alter table workspace_sessions add column schema_version integer not null default 2;
alter table panel_states add column panel_instance_key text not null default '';
alter table panel_states add column focused integer not null default 0 check (focused in (0, 1));
alter table panel_states add column panel_state_json text check (panel_state_json is null or json_valid(panel_state_json));
```

```rust
pub fn second_migration_sql() -> &'static str {
    include_str!("../migrations/0002_workspace_shell_session.sql")
}
```

```rust
async fn connect_and_migrate_all() -> SqliteConnection {
    let mut connection = connect_sqlite("sqlite::memory:")
        .await
        .expect("in-memory sqlite should open");
    sqlx::raw_sql(initial_migration_sql())
        .execute(&mut connection)
        .await
        .expect("initial migration should execute");
    sqlx::raw_sql(second_migration_sql())
        .execute(&mut connection)
        .await
        .expect("second migration should execute");
    connection
}
```

- [ ] **Step 4: Run the DB crate tests to verify they pass**

Run: `cargo test -p db -- --nocapture`

Expected: PASS with the existing schema checks and the new second-migration coverage.

- [ ] **Step 5: Commit**

```bash
git add crates/db/migrations/0002_workspace_shell_session.sql crates/db/src/db.rs crates/db/tests/db_tests.rs
git commit -m "feat: add schema-versioned workspace session persistence"
```

## Task 7: Replace the Fake Shell With Runtime-Backed Workbench Composition

**Files:**
- Modify: `crates/app/src/main.rs`
- Modify: `crates/app_ui/src/app_ui.rs`
- Modify: `crates/ui/src/ui.rs`
- Test: `crates/app/tests/bootstrap_runtime_tests.rs`

- [ ] **Step 1: Write the failing bootstrap test**

```rust
use app::boot_message;

#[test]
fn boot_message_references_the_workbench_kernel_shell() {
    assert_eq!(boot_message(), "starting Zed Workbench Kernel");
}
```

```rust
use workspace::WorkspaceController;
use app_ui::AppFrame;

#[test]
fn app_frame_no_longer_exposes_fake_top_bar_state() {
    let frame = AppFrame::new(WorkspaceController::new("session-1"));
    assert_eq!(frame.title_text(), "Zed Workbench Kernel");
}
```

- [ ] **Step 2: Run the app test to verify it fails**

Run: `cargo test -p app boot_message_references_the_workbench_kernel_shell -- --nocapture`

Expected: FAIL because the title still says `Zed-Style Desktop Framework`.

- [ ] **Step 3: Write the minimal shell composition changes**

```rust
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
}
```

```rust
pub struct WorkbenchShell {
    pub left_dock: AnyElement,
    pub center_dock: AnyElement,
    pub right_dock: Option<AnyElement>,
    pub bottom_dock: Option<AnyElement>,
    pub status_bar: AnyElement,
}

pub fn render_shell(shell: WorkbenchShell) -> impl IntoElement {
    div()
        .size_full()
        .flex()
        .flex_col()
        .child(
            div()
                .flex_1()
                .min_h_0()
                .flex()
                .child(shell.left_dock)
                .child(shell.center_dock)
                .when_some(shell.right_dock, |this, right| this.child(right)),
        )
        .when_some(shell.bottom_dock, |this, bottom| this.child(bottom))
        .child(shell.status_bar)
}
```

```rust
WindowOptions {
    titlebar: Some(TitlebarOptions {
        title: None,
        appears_transparent: true,
        traffic_light_position: None,
    }),
    window_decorations: Some(WindowDecorations::Client),
    app_id: Some("com.workspace.zed_workbench_kernel".to_owned()),
    ..Default::default()
}
```

- [ ] **Step 4: Run the app, app_ui, and ui tests to verify they pass**

Run: `cargo test -p app -p app_ui -p ui -- --nocapture`

Expected: PASS, with no references left to fake top bar labels, fake dock cards, or fake boot notifications.

- [ ] **Step 5: Commit**

```bash
git add crates/app/src/main.rs crates/app_ui/src/app_ui.rs crates/ui/src/ui.rs crates/app/tests/bootstrap_runtime_tests.rs
git commit -m "feat: render runtime-backed workbench shell"
```

## Task 8: Run Workspace-Wide Verification and Update Tech Documentation

**Files:**
- Modify: `tech.md`

- [ ] **Step 1: Write the documentation diff**

```md
## Runtime Flow

1. `app` boots the Zed-style shell with client-side window decorations.
2. `actions`, `commands`, `menu`, `keymap`, and `command_palette` dispatch typed actions into `workspace`.
3. `workspace` owns panel type registration, singleton instance lifecycle, dock layout, and restore-safe serialization.
4. `app_ui` and `ui` render dock hosts from runtime state rather than fake placeholders.
5. `welcome` is the first built-in singleton panel used to validate the real shell lifecycle.
```

- [ ] **Step 2: Run the repository verification script**

Run: `powershell -ExecutionPolicy Bypass -File .\script\verify.ps1`

Expected: PASS with `cargo check`, tests for the touched crates, and no stale placeholder-shell behavior left in docs.

- [ ] **Step 3: Run focused high-risk verification**

Run: `cargo test -p panel -p dock -p workspace -p actions -p commands -p welcome -p db -- --nocapture`

Expected: PASS with coverage for singleton lifecycle, dock movement, typed actions, restore degradation, and migration constraints.

- [ ] **Step 4: Commit the verification and documentation sync**

```bash
git add tech.md
git commit -m "docs: sync tech spec with workbench kernel runtime"
```

- [ ] **Step 5: Prepare rollback notes in the PR description**

```md
Rollback:
- revert commits back to the last fake-shell baseline
- keep `0002_workspace_shell_session.sql` disabled from runtime reads if restore regressions appear
- fall back to default shell layout with restore skipped
```

## Coverage Check

- Panel identity, singleton semantics, close/hide behavior, and restore mapping are covered by Tasks 1 and 4.
- Persistence compatibility, schema versioning, degraded restore, and DB migration are covered by Tasks 4 and 6.
- `foundation`, `paths`, and `actions` responsibilities are reflected in the file structure and Task 3, with `tech.md` synchronized in Task 8.
- Explorer/status/notifications special-surface behavior is implemented by the runtime-backed shell in Task 7 and validated through `workspace` and `app_ui`.
- Verification, rollback, and documentation sync are explicitly covered by Tasks 6 through 8.
