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

- `app`
- `app_ui`
- `workspace`
- `dock`
- `panel`
- `menu`
- `commands`
- `command_palette`
- `status_bar`
- `notifications`
- `theme`
- `keymap`
- `settings`
- `assets`
- `db`

Responsibilities:

- window bootstrap
- title bar and menu integration
- workbench runtime
- panel registry and lifecycle
- dock layout
- action dispatch
- state persistence
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

- explorer host
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

- view/entity handle
- focus handle
- icon/title metadata
- save/restore hooks
- dock preference

Business modules register panel types. The workspace creates and manages panel instances.

### 5. Module Registration

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

### Required Action Semantics

Shell-level panel behavior should support these operations:

- `OpenPanel(panel_id)`
- `FocusPanel(panel_id)`
- `TogglePanel(panel_id)`
- `ClosePanel(panel_id)`
- `MovePanel(panel_id, dock_position)`

Semantics:

- `OpenPanel` creates if missing, shows if hidden, activates, and focuses.
- `FocusPanel` only changes active/focused state.
- `TogglePanel` hides if already active under the chosen toggle rule; otherwise ensures the panel is open and focused.
- `ClosePanel` hides or removes according to panel policy.
- `MovePanel` updates dock placement and marks state dirty for persistence.

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
- menu or action can open `welcome`
- `welcome` appears in its dock as a real panel instance
- `welcome` can be focused and toggled
- session close/reopen restores dock placement, visibility, and focus

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

### Phase 2: Integrate Zed-Style Window Shell

- align window options with Zed-style shell behavior
- integrate title bar, application menu placement, and platform window controls
- move menu/titlebar responsibility out of content rendering

### Phase 3: Build Real Panel Runtime

- upgrade `panel`
- upgrade `workspace`
- upgrade `dock`
- connect menu/commands/keymap to runtime behavior
- render docks from runtime state

### Phase 4: Validate With `welcome`

- register `welcome` as a real panel type
- open via menu/action
- mount into dock
- persist and restore session state

### Phase 5: Enable Future Business Modules

- document module registration conventions
- use `welcome` as the canonical sample
- later integrate modules such as `stock_trading` through the same protocol

## Acceptance Criteria

The design is considered implemented correctly when all of the following are true:

- the top shell is no longer a fake content header
- the app contains a real title bar, menu, and workbench shell hierarchy
- at least one panel type is registered through the runtime
- a registered panel can be created, shown, focused, toggled, and restored
- dock placement is real runtime state, not hardcoded strings
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
