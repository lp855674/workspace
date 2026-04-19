# Zed-Style Rust Desktop Framework Design

## Summary

Build a standalone Rust desktop application workspace in `E:\code\workspace` that follows Zed's framework construction logic while remaining independent from the Zed repository. The framework should prioritize reuse of Zed's open-sourced crates and patterns, and copy only the smallest necessary glue code when a direct dependency would pull in too much unrelated complexity.

The first phase targets a medium-fidelity app shell rather than a full Zed clone. It should provide a single-window, single-workspace desktop host with:

- GPUI-based application startup and window lifecycle
- shell layout with sidebar, center area, bottom panel area, status bar, and toast layer
- panel registration and lifecycle management
- unified action, menu, keymap, and command palette flow
- theme, settings, and keymap loading
- session and layout restoration
- local state persistence through `sqlx + SQLite`
- static feature-module registration for later business integration

This phase explicitly does not implement editor features, remote/collab systems, multi-workspace orchestration, or stock-trading business logic.

## Goals

- Build an independent desktop framework project rather than extending the Zed repository.
- Preserve the core framework ideas from Zed: app bootstrapping, service registration, workspace shell, action routing, and persistent UI state.
- Keep the architecture ready for future cross-platform support while verifying first on Windows.
- Make later business modules attach through explicit registration points instead of editing the framework core.
- Finish the implementation with a project-level technical specification in `tech.md`.

## Non-Goals

- Reproducing the full Zed product architecture
- Implementing multi-window or multi-workspace restoration flows in phase one
- Building editor, project tree, LSP, extension marketplace, or remote features
- Designing a plugin hot-loading system
- Storing business-domain trading data in the framework database
- Creating packaging, installer, or release automation in phase one

## Project Shape

Start as a standalone workspace with a small number of crates. Avoid mirroring Zed's crate count.

Recommended structure:

- `crates/app`
  - executable entry point
  - GPUI application startup
  - global service wiring
  - main window creation
- `crates/shell`
  - root desktop shell view
  - sidebar/center/bottom/status/toast regions
  - shell composition and visual host logic
- `crates/workspace`
  - workspace controller
  - panel registry and panel lifecycle
  - layout state and restoration
  - command routing to workspace-owned UI
- `crates/foundation`
  - shared actions
  - settings and theme glue
  - serialization models
  - command and module descriptors
- `crates/state_db`
  - SQLite connection lifecycle
  - sqlx migrations
  - repositories for framework state
- `assets/`
  - default keymaps
  - themes
  - icons
  - default config assets
- `docs/superpowers/specs/`
  - design and planning documents
- `tech.md`
  - project technical source of truth after implementation

## Dependency Strategy

### Reuse Directly Where Feasible

Prefer direct reuse of Zed open-source crates that form the UI foundation:

- `gpui`
- `gpui_macros`
- platform-specific GPUI crates needed by the chosen build

These crates provide the core application model and should remain the base instead of being reimplemented.

### Reference and Rebuild at the Shell Layer

For Zed crates such as `workspace`, `ui`, `theme`, `menu`, `panel`, and title-bar related modules, prefer extracting their organizing ideas rather than importing them wholesale. In Zed they are tied to editor, project, settings, and product-wide assumptions. Pulling them in directly would force large unrelated dependencies.

The new framework should therefore:

- follow Zed's composition and lifecycle patterns
- preserve similar responsibilities and boundaries
- reimplement a focused subset with project-local types

### Copy Minimal Glue Code Only When Necessary

Copy narrowly-scoped glue logic only when:

- the logic is foundational rather than product-specific
- there is no clean standalone crate boundary available
- recreating it from scratch would add risk without meaningful benefit

Examples of acceptable copied logic:

- compact layout persistence models
- panel descriptor patterns
- toast host wiring
- small action registration helpers

Do not copy editor-, project-, extension-, or collab-specific logic into phase one.

## Architecture Overview

The framework should be organized as four layers.

### 1. App Layer

Owns process startup and GPUI initialization.

Responsibilities:

- start `gpui::Application`
- initialize global registries and services
- open the main window
- mount the root view
- load startup configuration and initial persisted state

The app layer must not contain feature-specific UI logic.

### 2. Shell Layer

Owns the main application frame.

Responsibilities:

- compose the root desktop layout
- host sidebar, center view, bottom dock, status bar, and toast layer
- expose stable insertion points for workspace and feature modules
- remain agnostic to business-module internals

The shell is the permanent host of the application and should not become a feature dump.

### 3. Workspace Layer

Owns the primary runtime state for the desktop host.

Responsibilities:

- register panel types
- create panel instances
- show, hide, focus, and restore panels
- coordinate workspace-local actions
- persist layout and session state
- provide the current workspace context to commands and modules

This is the main state center for phase one.

### 4. Foundation Layer

Owns cross-cutting definitions and stable contracts.

Responsibilities:

- command descriptors
- action definitions
- settings/theme interfaces
- serialization models
- module registration interfaces
- shared identifiers and enums

This layer should stay dependency-light so other crates can build on it without circular coupling.

## Runtime Flow

The startup flow should be:

1. `crates/app` starts the GPUI application.
2. Global services initialize:
   - settings registry
   - theme registry
   - command registry
   - keymap loader
   - notification/toast service
   - state database
3. `AppServices` is constructed as the shared service hub.
4. The main window opens.
5. `AppRoot` mounts the root shell.
6. `ShellView` composes the fixed desktop regions.
7. `WorkspaceController` restores persisted state and activates default UI.
8. Registered feature modules attach their panels, commands, and optional shell contributions.

This keeps the framework boot flow deterministic and makes later modules attach after the host is alive.

## Core Components

### `AppRoot`

Purpose:

- top-level composition entry
- injects service handles into the root shell

Constraints:

- no business state
- no panel ownership

### `ShellView`

Purpose:

- visual host for the main frame

Regions:

- left sidebar
- center content area
- bottom panel area
- status bar
- toast/notification overlay

Constraints:

- manages layout presentation
- does not own long-lived feature logic

### `WorkspaceController`

Purpose:

- central runtime coordinator for the shell

Responsibilities:

- panel registration
- panel creation and lookup
- visibility and focus transitions
- layout restoration
- workspace-scoped command routing
- session save/restore

This is the most important control point in phase one.

### `CommandRegistry`

Purpose:

- define a single source of truth for logical commands

Consumers:

- menus
- keyboard shortcuts
- command palette
- panel-local triggers

This avoids each surface creating its own command path.

### `SettingsStore`

Purpose:

- manage app settings and user-overridable configuration

Data managed by file:

- application preferences
- theme selection
- keymap overrides

### `ThemeManager`

Purpose:

- load and switch themes
- expose active theme values to the shell and modules

### `NotificationService`

Purpose:

- send toast and notification events to the shell
- optionally persist lightweight history for later restoration/debugging

## Module Registration Model

Business or product modules should integrate through a static registration interface rather than editing the host core.

### `FeatureModule`

Each module should be able to:

- register panels
- register commands/actions
- register default keybindings
- optionally contribute a status bar item
- perform startup initialization against `AppServices`

Phase one uses static module registration at startup. Dynamic loading is out of scope.

### `PanelDescriptor`

Each panel should provide:

- `panel_id`
- title
- default dock position
- default visibility
- focusability
- restorable flag

Suggested dock positions:

- `Left`
- `Center`
- `Bottom`
- `Right`

The first phase only needs the docks required by the shell layout. Additional positions can exist in the model even if not all are rendered initially.

### `PanelRegistry`

Responsibilities:

- map `panel_id` to descriptor and factory
- lazily construct panel instances when needed
- decouple shell/workspace code from concrete panel types

## Data Models

### `WorkspaceState`

Runtime state should include:

- registered panels
- visible panels
- focused panel
- dock/layout state
- recent command context
- current theme/settings snapshot as needed for rendering

It should contain runtime handles and ephemeral state that must not be serialized directly.

### `SerializedWorkspaceState`

Persistent state should include only restore-relevant data:

- visible panel ids
- focused panel id
- dock placement data
- simple layout settings
- last active center content context if applicable

### `CommandDescriptor`

Each command should define:

- stable `command_id`
- human-readable title
- category
- optional default keybinding
- enablement conditions
- invocation handler target

One descriptor should be reusable by menus, keymaps, and command palette.

### `AppServices`

Central service bundle should include at least:

- `workspace_controller`
- `settings_store`
- `theme_manager`
- `command_registry`
- `notification_service`
- `state_db`

This provides a stable integration surface for future modules.

## Persistence Design

Persistence is split between editable files and local database state.

### File-Based Configuration

Use files for user-editable configuration:

- `settings.json`
- `keymap.json`
- theme selection or theme override files

Rationale:

- easy to inspect and edit
- mirrors user-facing configuration patterns
- avoids forcing all preferences into SQL rows

### SQLite State Database

Use `SQLite` via `sqlx` for local framework runtime state.

Rationale:

- structured queryable persistence
- migration support
- clear boundary between config and runtime/session state
- better long-term fit than ad hoc JSON blobs for host state

The database is local-only and not a business backend.

### `state_db` Responsibilities

The `state_db` crate/module should own:

- opening the SQLite database
- connection pool management
- migration execution
- repository APIs
- serialization boundaries for persisted runtime state

No other crate should write raw SQL directly.

### Phase-One Tables

Initial schema should stay narrow and host-focused:

- `workspace_sessions`
- `panel_states`
- `command_history`
- `notifications`
- `module_state`

Possible contents:

- `workspace_sessions`
  - session id
  - timestamps
  - top-level workspace metadata
- `panel_states`
  - session id
  - panel id
  - dock
  - visibility
  - focus order or last active marker
- `command_history`
  - command id
  - invocation timestamp
  - optional workspace context
- `notifications`
  - notification id
  - level
  - message
  - created at
- `module_state`
  - module id
  - state key
  - serialized payload
  - updated at

Feature modules that need local persistence must use host-provided repository interfaces rather than creating their own arbitrary tables in phase one.

## Platform Strategy

The code structure should preserve future cross-platform support, but phase-one validation targets Windows first.

Implications:

- platform-sensitive paths should go through an abstraction
- menu/title-bar behavior should allow future platform branches
- avoid Windows-only assumptions in core state and shell models

The framework does not need to fully validate macOS/Linux in phase one, but it must not be structured in a way that prevents later adaptation.

## Must-Have Deliverables for Phase One

- A runnable desktop application on Windows
- A main shell with:
  - sidebar
  - center content area
  - bottom panel area
  - status bar
  - toast layer
- A panel system with registration, show/hide, focus, and restore
- A unified command system used by menu, keymap, and command palette
- Theme loading and switching
- Settings loading
- Keymap loading
- Session/layout restoration
- A static module registration interface
- At least one demo feature module proving non-invasive integration
- A completed root-level `tech.md` after implementation

## Explicitly Deferred

- multi-workspace orchestration
- multi-window restoration complexity
- project tree and file/worktree management
- editor engine
- language tooling and LSP
- collaboration/remote systems
- hot-loaded plugins/extensions
- packaging/release automation
- business-domain stock trading features

## Error Handling

Follow the repository error-handling discipline:

- do not silently discard failures
- initialization failures should be graded into:
  - fatal startup failures
  - recoverable module/service failures
- configuration/theme/keymap/layout load failures must log and fall back to defaults
- feature-module startup failure must not take down the entire shell unless the module is declared required

Expected behavior:

- if the database fails to open, startup should fail with a clear error because persistence is part of the framework core
- if a non-critical module fails to register, the shell should still launch and surface the failure in logs and a user-visible notification when appropriate

## Testing Strategy

### Unit Tests

Cover:

- command registration
- layout-state transitions
- serialization/deserialization
- settings merge behavior
- repository logic against test SQLite instances

### GPUI-Level Tests

Cover:

- panel registration
- panel show/hide/focus flows
- command dispatch into workspace-owned handlers
- shell state reactions to notifications

### Integration Tests

Cover:

- startup with default config
- restoration from persisted workspace state
- theme/keymap/settings load path
- module registration path

### Manual Acceptance Tests

Verify:

- app launches on Windows
- shell renders all expected regions
- at least three panels can be toggled and focused
- commands fire consistently from menu, keymap, and command palette
- theme changes and layout state survive restart
- a demo module can be added without modifying host core flow

## Acceptance Criteria

Phase one is complete when all of the following are true:

- the app starts successfully on Windows
- the shell layout is present and functional
- at least three panels work through the shared panel system
- commands are unified across surfaces
- session/layout persistence works through `sqlx + SQLite`
- a demo module integrates through the module registration interface
- the resulting architecture is documented in `tech.md`

## Risks and Mitigations

### Risk: Pulling Too Much From Zed

Mitigation:

- prefer pattern reuse over crate-level copying for shell/workspace layers
- define local interfaces first, then adapt borrowed logic to those interfaces

### Risk: GPUI and Platform Complexity on Windows

Mitigation:

- keep phase-one shell minimal
- validate startup and rendering early before expanding the panel system

### Risk: Overdesigning a Plugin System

Mitigation:

- static startup registration only
- no dynamic loading in phase one

### Risk: State Schema Sprawl

Mitigation:

- keep schema limited to host-state tables
- forbid ad hoc SQL outside `state_db`

## Implementation Guidance

When implementation begins, sequence the work roughly as:

1. workspace and crate scaffolding
2. app startup with GPUI window
3. shell layout
4. command registry and action wiring
5. panel registry and controller
6. settings/theme/keymap loading
7. `sqlx + SQLite` state database with migrations
8. session restoration
9. demo feature module
10. `tech.md` completion and verification

This ordering keeps visible progress early while preserving architecture boundaries.
