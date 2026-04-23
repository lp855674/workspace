# Zed Workbench Kernel Design

## Goal

Rebuild the current `workspace` app from a fake Zed-like shell into a real Zed-style workbench kernel.

The target is not a pixel-perfect fork of Zed and not a demo page that imitates an IDE. The target is a working shell with real runtime behavior:

- Zed-style title bar, application menu, and window controls
- real workspace runtime
- real dock and panel lifecycle
- real module registration and panel opening
- real focus/toggle/open behavior
- real session persistence and restore

This shell must become the stable base that future business modules can attach to, including modules shaped like `stock_trading`.

## Document Precedence

This spec supersedes older placeholder-shell descriptions in [tech.md](/E:/code/workspace/tech.md), especially statements that:

- place fake menu bar behavior inside `ui`
- treat placeholder shell regions as intended runtime surfaces
- describe the current fake shell as the target architecture

During implementation, `tech.md` must be updated to match the shipped runtime. Until then, this spec is the design source of truth for the workbench kernel rewrite.

## Problem Statement

The current implementation mixes shell rendering, fake UI placeholders, and runtime state in the wrong layer.

Current problems:

- `crates/ui` renders a fake top bar instead of a real title bar and menu system.
- center content is a self-descriptive placeholder card rather than a real empty workbench surface.
- right dock and bottom dock are fake demo surfaces instead of real dock containers.
- `workspace`, `dock`, and `panel` only model static metadata and visibility, not real panel instances and lifecycle.
- menu and commands are registered, but they are not connected to real panel open/toggle/focus behavior.
- status bar and notifications currently contain demo content, not real framework-level contributions.

The result looks like a shell but does not behave like one.

## Design Principles

1. UI emptiness is acceptable; fake functionality is not.
2. Workbench behavior must live in runtime crates, not in view-only shell code.
3. Business modules must plug into the shell through stable registration contracts.
4. The shell should follow Zed's architecture patterns where practical, without importing all of Zed's workspace complexity.
5. The first version must make the core lifecycle real before adding more panels or business features.

## Scope

### In Scope

- replace fake header with Zed-style title bar, menu placement, and window controls
- convert the app into a real workbench shell
- build panel registration, creation, open, toggle, focus, close, and dock placement behavior
- provide dock containers for `left`, `center`, `right`, and `bottom`
- persist and restore shell session state
- use `welcome` as the first real module to validate the system end-to-end

### Out of Scope

- full Zed workspace feature parity
- complex editor pane splitting
- drag-and-drop rearrangement
- multi-window workspaces
- collaboration and remote features
- complete command palette parity with Zed
- business modules beyond the validation path needed for `welcome`

## Reference Baseline From Zed

The design should align with Zed's architectural direction in these areas:

- top-level window configuration from Zed's window options pattern
- title bar and platform window controls from Zed's title bar stack
- application menu being part of the real window shell, not a fake workbench row
- panel lifecycle being driven by the workspace runtime
- business panels implementing a panel contract and being mounted by the workspace

The design explicitly does **not** copy Zed's entire `workspace` implementation. This repository keeps its own `workspace`, `dock`, and `panel` crates, but their interfaces and responsibilities are reshaped to match the same runtime model.

## Architecture

The system is divided into two layers.

### Foundation Layer

Framework crates:

- `foundation`
- `app`
- `app_ui`
- `workspace`
- `dock`
- `panel`
- `actions`
- `menu`
- `commands`
- `command_palette`
- `status_bar`
- `notifications`
- `theme`
- `keymap`
- `settings`
- `paths`
- `assets`
- `db`

Responsibilities:

- shared base types, errors, and serialization helpers
- window bootstrap
- title bar and menu integration
- workbench runtime
- panel registry and lifecycle
- dock layout
- action types and dispatch
- state persistence
- config and storage path resolution
- shell-level contribution surfaces

### Business Layer

Business crates:

- `welcome`
- future modules such as `stock_trading`

Responsibilities:

- business services
- business actions
- business panel implementations
- optional menu and status contributions
- module initialization

Business modules must not own global layout management.

## Target Runtime Model

### 1. Workbench Frame

`app_ui` becomes a composition host, not a fake workbench renderer.

It renders:

- title bar host
- workbench body host
- status bar host

The workbench body host renders:

- left dock host
- center dock host
- right dock host
- bottom dock host

It consumes runtime state from `workspace`; it does not invent panel strings or placeholder cards.

### 2. Workspace Runtime

`workspace` becomes the main runtime controller.

It owns:

- registered panel types
- live panel instances
- dock layout state
- focused panel state
- visible panel state
- active tab per dock
- session state
- action execution for shell-level panel behavior

Core operations:

- register panel type
- ensure panel created
- open panel
- focus panel
- toggle panel
- close panel
- move panel to a dock
- restore session
- serialize session

### 3. Dock Runtime

`dock` becomes a layout runtime, not just a placement enum plus visible list.

It owns:

- dock containers for `left`, `center`, `right`, `bottom`
- per-dock tab collections
- active tab per dock
- visibility per dock
- size per dock

The dock layer decides where panel instances live and which one is active in each dock.

### 4. Panel Runtime Contract

`panel` becomes the registration and instance protocol.

Two concepts are needed:

#### Panel Type

Static registration contract for a panel kind:

- `id`
- `title`
- `default_dock`
- `toggle_action_id`
- `create/load` callback

#### Panel Instance

Live panel runtime object:

- instance identity
- view/entity handle
- focus handle
- icon/title metadata
- save/restore hooks
- dock preference

Business modules register panel types. The workspace creates and manages panel instances.

### 5. Identity and Lifetime Model

The panel model must explicitly separate type identity from instance identity.

#### Identity Terms

- `panel_type_id`: stable identifier for a panel kind, for example `welcome.panel`
- `panel_instance_id`: runtime identity for one concrete mounted instance
- `panel_instance_key`: optional stable restore key used to map serialized state back to an instance across sessions

For the MVP:

- all panels are singleton by default
- `panel_type_id` is the user-facing identity used by menu, keymap, command palette, and shell actions
- each singleton panel still receives an internal `panel_instance_id`
- for singleton panels, `panel_instance_key` defaults to the `panel_type_id`

For future work:

- multi-instance panels may opt in explicitly
- multi-instance panels must provide an instance creation policy and stable restore key strategy

#### Creation Semantics

`ensure panel created` means:

- singleton panel:
  - if an instance for `panel_type_id` exists, reuse it
  - otherwise create one instance and register its `panel_instance_id`
- multi-instance panel:
  - resolve the requested `panel_instance_key`
  - if a matching instance exists, reuse it
  - otherwise create a new instance for that key

The MVP does not require multi-instance support, but the runtime APIs must leave room for it by not collapsing type id and instance id into the same field.

#### Lifetime Semantics

Default shell policy:

- `OpenPanel` creates or reuses an instance, makes it visible, activates it, and focuses it
- `TogglePanel` hides the panel if it is currently active and visible; otherwise it behaves like `OpenPanel`
- `ClosePanel` hides the instance by default; it does not destroy singleton instances in the MVP
- `DestroyPanel` is an internal runtime operation, not an MVP user-facing action

Panel types may later opt into destroy-on-close behavior, but the default policy for the first implementation is hide-on-close to keep restore behavior predictable.

#### Restore Mapping

Serialized session records must contain enough information to restore instances:

- `panel_type_id`
- `panel_instance_key`
- dock placement
- visibility
- active/focused state
- panel-owned state blob

Restore resolves records in this order:

1. find registered panel type by `panel_type_id`
2. resolve existing instance by `panel_instance_key`
3. create instance if needed
4. restore shell-owned state
5. pass panel-owned blob to the panel restore hook

### 6. Module Registration

`module` stays as the integration entry point, but it must grow beyond metadata-only registration.

Each module should be able to register:

- panel types
- actions
- menu contributions
- status bar contributions
- startup hooks

Conflict detection remains in the module layer, but actual lifecycle belongs to `workspace`.

## Action and Command Flow

The real shell flow is:

`menu/keymap/command_palette -> action -> workspace runtime -> dock manager -> panel open/focus/toggle -> frame rerender`

This replaces the current string-based fake shell state.

### Action Contract

`actions` owns the shared action contract for the workspace.

Responsibilities:

- action identifiers and payload types
- action serialization shape for testing and persistence where needed
- shared dispatch contract between shell surfaces

Uniform action flow:

- `menu` resolves a menu item into an action
- `keymap` resolves a key chord into an action
- `command_palette` resolves a command selection into an action
- `commands` maps command descriptors to actions
- `workspace` executes the action

The shell must not let UI surfaces call runtime methods directly for panel lifecycle behavior. They must dispatch actions through the shared action model.

#### Action Payload Rules

- shell-level panel actions should be represented as typed actions, not free-form strings
- action payloads used in tests or restore paths must be serializable
- command-to-action mapping must be explicit in `commands`

#### Surface Priority Rules

Priority applies only to input resolution, not to action semantics:

- keymap conflicts are resolved by keymap context rules
- menu invocation always dispatches its configured action
- command palette always dispatches its selected command's mapped action

After dispatch, all three surfaces share the same runtime action execution path.

### Required Action Semantics

Shell-level panel behavior should support these operations:

- `OpenPanel(panel_type_id)`
- `FocusPanel(panel_type_id)`
- `TogglePanel(panel_type_id)`
- `ClosePanel(panel_type_id)`
- `MovePanel(panel_type_id, dock_position)`

Semantics:

- `OpenPanel` creates if missing, shows if hidden, activates, and focuses.
- `FocusPanel` only changes active/focused state.
- `TogglePanel` hides if already active under the chosen toggle rule; otherwise ensures the panel is open and focused.
- `ClosePanel` hides or removes according to panel policy.
- `MovePanel` updates dock placement and marks state dirty for persistence.

For the MVP, `panel_type_id` targets singleton panels. Future multi-instance actions may add an explicit `panel_instance_key`.

## Special Surfaces

The shell should avoid layout special cases where possible.

### Explorer

The left explorer surface is treated as a built-in panel type managed by the same runtime model as other panels.

That means:

- it lives in the left dock
- it has a registered panel type
- it can be shown, focused, hidden, restored, and persisted through the same runtime path

This avoids introducing a second layout system just for the left side of the shell.

### Status Bar

The status bar remains a fixed shell slot, but its contents are contribution-driven.

- the shell owns slot layout and rendering lifecycle
- foundation and business modules may contribute status items
- fake hardcoded demo items must be removed

### Notifications

Notifications remain a shell-level service, not a dock panel.

- the shell owns notification presentation and lifecycle
- modules may emit notifications
- startup demo notifications must be removed

## Session Persistence

Persistence is split into shell-owned state and panel-owned state.

### Shell-Owned Session State

Owned by `workspace` and persisted through the foundation layer:

- visible panels
- focused panel
- panel dock placement
- active tab per dock
- dock sizes
- shell layout state

### Panel-Owned State

Owned by each panel implementation:

- panel-specific UI state
- business state needed for restoration

Examples:

- `welcome`: local view mode or selected tab
- future `stock_trading`: selected symbol, order form values, local filters

The shell only stores opaque panel state blobs or delegates save/restore through the panel contract.

### Persistence Compatibility

Session persistence must be versioned and failure-tolerant.

Required persisted envelope fields:

- `schema_version`
- serialized shell-owned session state
- per-panel serialized records

#### Compatibility Rules

- unknown `panel_type_id`:
  - skip that panel record
  - continue restoring the rest of the session
- renamed or removed dock:
  - fall back to the panel type's default dock
- invalid dock placement for the current panel type:
  - fall back to the panel type's default dock
- panel-owned state blob fails to deserialize:
  - create the panel with default state
  - mark restore as degraded
- incompatible session schema:
  - skip restore for the incompatible portion
  - preserve app startup

#### Fallback UI Behavior

Restore failure must never block app startup.

If restore is degraded:

- the app opens with the remaining valid shell state
- missing or invalid panels are omitted
- invalid panel-owned state falls back to panel defaults
- the runtime may emit one framework-level restore warning notification, but must not spam one notification per failed record

#### Schema Version Policy

- shell session serialization uses an explicit schema version
- breaking persistence changes must bump the schema version
- migration may be added later, but the MVP must at least support version detection and safe fallback

## Current Crate Migration Plan

### `app`

Keep, but simplify responsibilities:

- bootstrap app
- configure window options
- initialize title bar and menus
- initialize runtime and modules

Remove:

- fake notifications
- fake status data
- demo shell state injection

### `app_ui`

Keep, but rewrite around composition:

- title bar host
- workbench host
- status bar host

Remove shell demo data fields like fake tabs, fake dock items, and empty-center explanatory content.

### `ui`

Shrink to shared layout/render helpers.

Remove:

- fake top bar
- empty-center welcome card
- fake right dock cards
- fake bottom log output
- explanatory copy pretending to be product behavior

### `workspace`

Rewrite into the real runtime center.

### `dock`

Rewrite from static state into live dock layout management.

### `panel`

Rewrite from plain descriptors into a real registration and instance protocol.

### `module`

Keep, but expand from metadata collection into module-level contribution registration.

### `actions`

Keep as the shared typed action contract layer.

It should own:

- action ids
- typed payloads
- serialization-friendly action envelopes for testing and command mapping

### `paths`

Keep as the single source of config, cache, and persistence paths.

It should own:

- session storage location
- panel state path resolution
- config file roots used by restore and persistence

### `foundation`

Keep as the shared low-level crate for:

- common error types
- serialization helpers
- shared identifiers and base value objects
- schema version helpers used by persistence

### `welcome`

Keep as the first real validation module.

It must become the first true panel type registered into the shell, not a fake center placeholder.

## Minimum Viable Version

The first real version must provide:

1. Zed-style top shell
2. real workspace runtime
3. real dock mechanism
4. one real module (`welcome`)
5. real session persistence for shell state

### MVP Must Demonstrate

- app boots into a Zed-style base shell
- menu, keymap, and command palette can each dispatch the same action model to open `welcome`
- `welcome` appears in its dock as a real panel instance
- `welcome` is singleton in the MVP
- the first `TogglePanel(welcome.panel)` opens and focuses `welcome`
- the second consecutive `TogglePanel(welcome.panel)` hides `welcome`
- `OpenPanel(welcome.panel)` after a hide re-shows the same singleton instance
- moving `welcome` to another dock and restoring a session preserves dock placement, visibility, and focus when the serialized state is valid
- missing or invalid `welcome` state still allows startup with a default `welcome` instance or no `welcome` instance, depending on whether the shell record says it should be visible

### MVP Can Delay

- pane splits
- drag-and-drop
- advanced resize interactions
- multi-workspace behavior
- full command palette feature set
- additional business modules

## Non-Goals and Guardrails

Do not:

- build another fake workbench layer in `ui`
- keep explanatory placeholder cards in the center surface
- introduce a second business-owned layout manager like `stock_trading::PanelManager` into the foundation layer
- copy all of Zed's `workspace` complexity into this repository

Business modules may coordinate business events, but layout and panel lifecycle stay in the foundation layer.

## Recommended Implementation Phases

### Phase 1: Remove Fake Shell

- delete fake header
- delete fake right dock
- delete fake bottom demo panel
- delete fake welcome card
- leave only real shell hosts
- update `tech.md` to remove placeholder-shell claims once this phase lands

### Phase 2: Integrate Zed-Style Window Shell

- align window options with Zed-style shell behavior
- integrate title bar, application menu placement, and platform window controls
- move menu/titlebar responsibility out of content rendering
- verify top shell behavior with focused UI checks in `app` and `app_ui`

### Phase 3: Build Real Panel Runtime

- upgrade `panel`
- upgrade `workspace`
- upgrade `dock`
- upgrade `actions` and `commands` mapping
- connect menu/commands/keymap to runtime behavior
- render docks from runtime state
- add or adjust persistence schema and any needed DB migration
- add unit tests for register/open/toggle/focus/close/move transitions

### Phase 4: Validate With `welcome`

- register `welcome` as a real panel type
- open via menu/action
- mount into dock
- persist and restore session state
- add integration tests for restore degradation paths

### Phase 5: Enable Future Business Modules

- document module registration conventions
- use `welcome` as the canonical sample
- later integrate modules such as `stock_trading` through the same protocol

## Verification and Rollback

### Verification Matrix

At minimum, implementation must verify:

- `panel`:
  - identity and singleton lifecycle behavior
- `dock`:
  - dock placement, active tab, and visibility transitions
- `workspace`:
  - register/open/toggle/focus/close/move/restore state transitions
- `actions` and `commands`:
  - command-to-action mapping and dispatch
- `welcome`:
  - full registration-to-render-to-restore integration
- `db`:
  - any schema change used for session persistence

Recommended checks:

- `cargo test -p panel -p dock -p workspace -p welcome`
- `cargo test -p commands -p actions`
- `cargo test -p db` when persistence schema changes
- `cargo check --workspace`

### Rollback Strategy

If a phase introduces unstable restore or shell regressions:

- disable new restore reads through a version or feature gate and fall back to default shell state
- preserve startup even when session restore is skipped
- keep migration changes backward-safe where possible
- revert to the prior shell composition only at the phase boundary, not by mixing old fake UI with the new runtime

The first safe fallback is always:

- app starts
- default shell layout loads
- no persisted session is applied

## Acceptance Criteria

The design is considered implemented correctly when all of the following are true:

- the top shell is no longer a fake content header
- the app contains a real title bar, menu, and workbench shell hierarchy
- at least one panel type is registered through the runtime
- `welcome` is defined as an MVP singleton panel type
- a registered panel can be created, shown, focused, toggled, hidden, and restored through the shared action path
- dock placement is real runtime state, not hardcoded strings
- menu, keymap, and command palette all dispatch the same `welcome` open/toggle behavior through the action model
- restore succeeds when state is valid and degrades safely when state is missing, unknown, or invalid
- `welcome` validates the full registration-to-render-to-restore flow

## Risks

### Risk 1: Keeping Fake UI While Adding Runtime

If the fake shell remains, the architecture will stay confused and later work will become harder.

Mitigation:

- delete fake content first

### Risk 2: Over-copying Zed

Importing too much of Zed's `workspace` stack will explode scope and dependencies.

Mitigation:

- borrow patterns and shell pieces, not the full workspace crate

### Risk 3: Business Modules Owning Layout

If business modules keep private panel managers, the app will end up with two layout systems.

Mitigation:

- require all panel lifecycle and dock placement to go through foundation runtime

### Risk 4: Starting Business Work Before Runtime Is Real

Adding more panels before the base lifecycle is finished will cause churn and rework.

Mitigation:

- validate with `welcome` first

## Decision

Proceed by building a real Zed-style workbench kernel in this repository.

Use Zed as the structural reference for:

- window shell
- title bar
- menu placement
- platform window controls
- panel lifecycle direction

Keep this repository's own foundation crates, but reshape them into a real workbench runtime before attaching further business modules.
