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

Start as a standalone workspace that follows Zed's framework style more closely at the base-system layer, while still excluding editor-, project-, language-, remote-, and collab-specific systems from phase one.

Recommended structure:

- `crates/app`
  - executable entry point
  - GPUI application startup
  - global service wiring
  - main window creation
- `crates/app_ui`
  - root desktop application frame
  - sidebar/center/bottom/status/toast regions
  - app frame composition and visual host logic
- `crates/workspace`
  - workspace controller
  - session state and restoration
  - workspace context and focus coordination
  - command routing to workspace-owned UI
- `crates/module`
  - feature module contracts
  - contribution model
  - module runtime and registration lifecycle
- `crates/panel`
  - panel traits
  - panel descriptors and registry
  - panel lifecycle semantics
- `crates/dock`
  - dock placement model
  - visible panel layout
  - dock restoration semantics
- `crates/ui`
  - reusable UI components and base containers
- `crates/theme`
  - theme model
  - tokens and active theme state
  - theme loading and switching
- `crates/settings`
  - settings model
  - defaults, loading, and overrides
- `crates/keymap`
  - keybinding model
  - default keymap and user overrides
- `crates/menu`
  - menu model
  - command and action bridging
- `crates/actions`
  - logical action definitions
- `crates/commands`
  - command registry
  - enablement and dispatch
- `crates/notifications`
  - toast and notification model
  - notification service and host glue
- `crates/command_palette`
  - command palette state and UI host
- `crates/status_bar`
  - status bar item contracts
  - status bar host
- `crates/foundation`
  - shared IDs and traits
  - contexts and common error types
  - serialization models
  - `AppServices`
- `crates/db`
  - SQLite connection lifecycle
  - sqlx migrations
  - repositories for framework state
- `crates/paths`
  - config/data/log/cache path conventions
- `crates/assets`
  - default themes, keymaps, fonts, icons, and asset loading
- `crates/welcome`
  - default center content
  - demo feature module for framework validation
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

## Missing Capability Check

The framework must explicitly account for the following capability lines so they do not disappear into unrelated crates:

- startup and lifecycle
- paths and assets
- workspace session state
- focus and interaction context
- actions, commands, keymap, menu, and command palette
- panel and dock separation
- notifications and status bar hosting
- module runtime and contributions
- migrations and repositories around `sqlx + SQLite`

Some of these become dedicated crates in phase one, while others may remain submodules if they stay narrow. They must still be named explicitly in the design.

## Architecture Overview

The framework should be organized as five layers.

### 1. App Layer

Owns process startup and GPUI initialization.

Responsibilities:

- start `gpui::Application`
- initialize global registries and services
- open the main window
- mount the root view
- load startup configuration and initial persisted state

The app layer must not contain feature-specific UI logic.

### 2. App Frame Layer

Owns the main application frame.

Responsibilities:

- compose the root desktop layout
- host sidebar, center view, bottom dock, status bar, and toast layer
- expose stable insertion points for workspace and feature modules
- remain agnostic to business-module internals

The application frame is the permanent host of the application and should not become a feature dump.

### 3. Workspace Layer

Owns the primary runtime state for the desktop host.

Responsibilities:

- coordinate panel, dock, focus, and session state
- coordinate workspace-local actions and contexts
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

### 5. Module Layer

Owns the framework's extension mechanism.

Responsibilities:

- define the feature-module interface
- define contribution types
- aggregate contributions from modules
- manage initialization order and failure handling
- isolate module-facing API from host implementation details

This layer should be one of the framework's stable open-source cores.

## Runtime Flow

The startup flow should be:

1. `crates/app` starts the GPUI application.
2. Global services initialize:
   - settings registry
   - theme registry
   - paths and asset loading
   - command registry
   - keymap loader
   - notification/toast service
   - state database
3. `AppServices` is constructed as the shared service hub.
4. Static feature modules are loaded through the module runtime.
5. Contributions are aggregated and routed to the owning subsystems.
6. The main window opens.
7. `AppRoot` mounts the root application frame.
8. `ShellView` composes the fixed desktop regions.
9. `WorkspaceController` restores persisted state and activates default UI.

This keeps the framework boot flow deterministic and makes later modules attach after the host is alive.

## Core Components

### `AppRoot`

Purpose:

- top-level composition entry
- injects service handles into the root application frame

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

- central runtime coordinator for the application frame

Responsibilities:

- panel and dock coordination
- visibility and focus transitions
- layout and session restoration
- workspace-scoped command routing
- session save/restore

This is the most important control point in phase one.

### `ModuleRuntime`

Purpose:

- load and initialize static feature modules
- aggregate module contributions
- apply ordering, conflict detection, and failure policy

Responsibilities:

- track module registration success and failure
- keep optional module failures from taking down the full host
- provide a stable path for future business modules to attach

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

Business or product modules should integrate through a dedicated module system rather than editing the host core.

### `FeatureModule`

Each module should expose:

- stable `module_id`
- human-readable name
- version
- required or optional status
- startup registration entrypoint

Each module should be able to:

- register panels
- register commands/actions
- register default keybindings
- optionally contribute a status bar item
- perform startup initialization against `AppServices`

Phase one uses static module registration at startup. Dynamic loading is out of scope.

### Contribution Types

A feature module may contribute:

- `PanelContribution`
- `CommandContribution`
- `KeymapContribution`
- `MenuContribution`
- `StatusBarContribution`
- `SettingsContribution`
- `NavigationContribution`
- `PersistenceContribution`

These contributions must be aggregated through `crates/module` and then routed to the systems that own them.

### Contribution Consumers

Each contribution is consumed by a host subsystem:

- panel contributions by `panel`, `dock`, and `workspace`
- command contributions by `actions`, `commands`, `menu`, `keymap`, and `command_palette`
- status bar contributions by `status_bar`
- settings contributions by `settings`
- persistence contributions by `db`
- navigation contributions by `workspace` and `app_ui`

### Why `crates/module` Must Exist

The module system is not a miscellaneous shared-types layer. It is the framework's extension mechanism and should be stable, explicit, and reusable as an open-source core.

It should therefore:

- avoid depending on host implementation details where possible
- expose stable module-facing interfaces
- centralize conflict detection
- own optional-versus-required failure policy

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
- `db`
- `paths`
- `asset_loader`
- `module_runtime`

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

### `db` Responsibilities

The `db` crate/module should own:

- opening the SQLite database
- connection pool management
- migration execution
- repository APIs
- serialization boundaries for persisted runtime state

No other crate should write raw SQL directly.

### Migrations and Repositories

The migration layer and the repository layer should be called out explicitly in implementation, not treated as incidental details.

- migrations define the framework-owned schema
- repositories define the only supported way for host systems and feature modules to persist framework state

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
- avoid Windows-only assumptions in core state and application-frame models

The framework does not need to fully validate macOS/Linux in phase one, but it must not be structured in a way that prevents later adaptation.

## Must-Have Deliverables for Phase One

- A runnable desktop application on Windows
- A main application frame with:
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
- A dedicated `crates/module` contribution system
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
- if a module contribution conflicts with an existing stable id, registration should fail clearly and identify the owning module and contribution kind

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
- contribution aggregation and conflict handling

### Manual Acceptance Tests

Verify:

- app launches on Windows
- application frame renders all expected regions
- at least three panels can be toggled and focused
- commands fire consistently from menu, keymap, and command palette
- theme changes and layout state survive restart
- a demo module can be added without modifying host core flow

## Acceptance Criteria

Phase one is complete when all of the following are true:

- the app starts successfully on Windows
- the application frame layout is present and functional
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
- forbid ad hoc SQL outside `db`

### Risk: Module API Instability

Mitigation:

- keep `crates/module` focused on contracts and contribution models
- avoid leaking host implementation details into module-facing APIs

## Implementation Guidance

When implementation begins, sequence the work roughly as:

1. workspace and crate scaffolding
2. app startup with GPUI window
3. paths and assets setup
4. application frame layout
5. actions, commands, menu, and keymap wiring
6. panel and dock systems
7. module system and contribution aggregation
8. settings/theme loading
9. `sqlx + SQLite` database with migrations and repositories
10. session restoration
11. demo feature module
12. `tech.md` completion and verification

This ordering keeps visible progress early while preserving architecture boundaries.
