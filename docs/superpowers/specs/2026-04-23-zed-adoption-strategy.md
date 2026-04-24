# Zed Adoption Strategy for the Base Workbench Framework

## Goal

Decide whether this repository should:

- fork or copy Zed source code and delete unneeded parts
- or keep the current workspace structure and rewrite the workbench kernel with Zed as a reference

This document records the decision and the implementation boundary.

## Relationship to the Workbench Kernel Spec

This document is the adoption strategy. It answers:

- how this repository should use Zed as a reference
- whether Zed source should become the implementation base
- which parts of Zed are relevant or irrelevant

The workbench kernel design spec answers a different question:

- what runtime model this repository should implement
- how `workspace`, `dock`, `panel`, `module`, and `welcome` should work together
- what behavior must be accepted before the kernel is considered real

The intended reading order is:

1. read this document to understand the Zed adoption boundary
2. read `2026-04-23-zed-workbench-kernel-design.md` to understand the target architecture
3. read the implementation plan to execute the work in small verified steps

If these documents appear to conflict, use this rule:

- this document controls whether to copy, fork, or reference Zed
- the workbench kernel spec controls this repository's runtime contracts

## Decision

Use Zed as a structural reference, but do **not** use Zed source as the codebase foundation.

The recommended approach is:

- keep this repository's crate boundaries
- rewrite the workbench kernel in this repository
- reference Zed for shell composition, title bar patterns, menu placement, and panel lifecycle direction
- avoid inheriting Zed's editor, project, pane, and collaboration architecture

In short:

> copy the architecture ideas, not the product codebase

## Why Not Fork Zed and Delete Code

Forking Zed looks faster at the beginning, but it is the wrong optimization for this repository.

### 1. The target is a framework, not a trimmed editor product

This repository is building a reusable host framework that future business modules can attach to.

Zed is an editor product with its own assumptions about:

- editor items
- project state
- pane management
- collaboration flows
- language tooling
- workspace complexity shaped by those features

If this repository starts from Zed and deletes features, the remaining architecture will still be biased toward Zed's product model instead of this repository's host framework model.

### 2. Deleting code is often slower than writing a small clean kernel

The hard part is not removing visible UI.

The hard part is understanding:

- hidden state ownership
- implicit lifecycle coupling
- assumptions between modules
- code paths that only make sense inside Zed's full product

That means "fork and delete" usually turns into:

- trace dependencies
- discover surprising breakage
- keep compatibility glue
- live with architecture that no longer fits

For a small workbench kernel, that is usually slower than writing the runtime directly.

### 3. This repository already has the right coarse-grained boundaries

The workspace already splits responsibility across crates such as:

- `app`
- `app_ui`
- `workspace`
- `dock`
- `panel`
- `module`
- `welcome`

That is a better starting point for a reusable shell than importing a large external codebase and trying to reshape it afterward.

### 4. Forking Zed would import the wrong long-term constraints

Even if the first shell boots faster, the codebase would inherit:

- Zed naming and state model
- Zed-specific abstractions
- Zed evolution pressure
- extra code that future business modules do not need

The result would be a framework that always feels like a modified editor rather than a purpose-built host.

## What To Learn From Zed

Zed is still useful as a reference implementation.

The right way to use it is to study how it solves shell-level problems, then implement those ideas with this repository's own contracts.

## Reference Checklist

When looking at Zed, keep the reference scope narrow.

### Look at these areas

- application startup shape
- window options
- title bar structure
- platform window control placement
- application menu mounting
- root shell composition
- high-level panel lifecycle direction
- separation between visual shell and workspace runtime

The output of this research should be notes or repository-native code, not copied source.

### Do not use these areas as implementation sources

- editor model
- project model
- pane splitting
- item lifecycle
- collaborative editing
- remote workspace flows
- language tooling
- complex command palette parity
- Zed-specific database or settings assumptions

### Reference rule

For every Zed-inspired implementation, be able to state the idea in repository terms.

Good:

- "The title bar belongs to the shell frame, not to fake content UI."
- "Panel focus is workspace runtime state."
- "Menu items dispatch typed actions."

Bad:

- "Zed has this type, so we should import the same abstraction."
- "Zed uses this workspace concept, so our framework should also have it."
- "This looks like Zed, so the architecture is probably correct."

### Safe reference areas

- top-level window options
- title bar composition
- native window controls placement
- application menu placement
- separation of shell frame from runtime state
- workspace-driven panel lifecycle direction

These are good sources of structural guidance.

### Things to copy as ideas, not code

- title bar and menu are part of the real shell, not fake content rows
- panel visibility and focus are runtime state, not demo strings
- the shell frame should be thin
- business surfaces should mount through contracts, not bespoke layout logic

## What Not To Import From Zed

The following areas should not become the foundation of this repository:

- editor implementation
- project model
- pane splitting model
- item model tied to editor tabs
- collaboration and remote flows
- language tooling integrations
- any workspace complexity that exists only to support Zed's product features

These are not shortcuts for a base workbench framework. They are unrelated product baggage.

## Source Code and License Boundary

This repository should not copy Zed source code as a shortcut.

The default policy is:

- study Zed behavior and architecture
- write repository-native implementations
- keep naming and contracts aligned with this repository's crates
- avoid copied files, copied modules, or large copied snippets

If a future change proposes copying any Zed code directly, it must be handled as an exception.

The exception must document:

- why the behavior cannot reasonably be reimplemented
- exactly which source file or snippet is being copied
- license compatibility
- attribution requirements
- maintenance cost
- how the copied code will be isolated from this repository's runtime contracts

Small implementation ideas do not need this process. Direct source copying does.

The practical rule is:

> learn from Zed, but make the resulting code belong to this repository

## Drift Signals

The project is drifting away from this decision if any of these appear:

- `ui` starts storing fake workspace state again
- `app_ui` invents panel strings or demo dock data instead of consuming runtime state
- business modules create private dock or panel managers
- shell lifecycle actions bypass the shared action model and call runtime internals directly
- `workspace` starts adopting editor/project/pane concepts only because Zed has them
- the implementation introduces Zed-like complexity before `welcome` proves the core lifecycle
- session restore assumes all historical panel types still exist
- status bar or notifications return to demo content instead of contribution-driven data
- a new abstraction is justified by "Zed does it" rather than by this repository's requirements
- a future business module must edit core shell code to appear in the workbench

If one of these signals appears, stop and re-check the kernel spec before continuing.

## Core Strategy

Build a small real workbench kernel in this repository, then use `welcome` as the first end-to-end validation module.

The strategy is:

1. keep the current crate structure
2. remove fake shell behavior
3. define a real panel contract
4. define a real dock runtime
5. define a real workspace runtime
6. attach menu, commands, keymap, and command palette through a shared action model
7. validate the full flow with `welcome`

This gives the repository a foundation it owns.

## Crate-by-Crate Guidance

### Crates That Can Reference Zed Patterns

#### `app`

Use Zed as a reference for:

- process startup
- window options
- native decorations and controls behavior
- root shell wiring

Do not import Zed's editor or product initialization chain.

#### `app_ui`

Use Zed as a reference for:

- frame shell composition
- title bar layering
- menu host placement
- shell hierarchy

`app_ui` should stay a composition host, not become a fake runtime source.

#### `ui`

Use only as a thin rendering/helper layer.

It may borrow layout ideas, but it should not own fake workspace data, fake tabs, or fake panel content.

### Crates That Must Be Defined by This Repository

#### `panel`

This crate must be owned here.

It should define:

- panel type identity
- panel instance identity
- singleton vs multi-instance policy
- create/load hooks
- save/restore hooks
- metadata needed by docks and runtime

This is one of the core framework assets. It must not be inherited from Zed.

#### `dock`

This crate must define this repository's layout runtime.

It should own:

- dock containers
- tab collections per dock
- active tab per dock
- visibility per dock
- size state per dock
- move/show/hide behavior

It should not adopt Zed's pane model unless this repository explicitly decides to build that later.

#### `workspace`

This crate must be the runtime center.

It should own:

- panel registry
- panel instance lifecycle
- open/focus/toggle/close semantics
- dock coordination
- focus state
- restore and serialization
- shell-level action execution

This is the most important crate to write directly.

#### `module`

This crate must define the module integration contract for this repository.

It should register:

- panels
- actions
- menu contributions
- status bar contributions
- startup hooks

It should remain the boundary between framework and business modules.

#### `welcome`

This crate should become the first real validation module.

It should prove:

- panel registration
- action registration
- panel creation
- focus and toggle behavior
- restore-safe state

`welcome` should stop acting like a fake placeholder.

### Crates That Should Be Minimal and Runtime-Aligned

#### `actions`

This crate should define the shared typed action model.

It should be the common contract used by:

- menu
- keymap
- command palette
- runtime dispatch

UI surfaces should not call runtime methods directly for panel lifecycle behavior.

#### `menu`

This crate can borrow shell/menu placement ideas from Zed, but it must target this repository's action contract.

#### `commands`

This crate should expose command descriptors that map to typed actions, not free-form strings.

#### `command_palette`

This crate should stay minimal in the first version.

It does not need Zed-level parity. It only needs enough behavior to trigger real framework actions.

#### `status_bar`

This crate should become a real contribution surface instead of demo content.

#### `notifications`

This crate should be a real shell service, not a fake dock surface.

#### `paths`

This crate should remain the source of shell/session/config path resolution and should be part of the design whenever persistence is specified.

#### `foundation`

This crate should remain the place for shared low-level types/helpers if needed, but it should not silently become a second runtime center.

## Recommended Build Order

Do not start from shell visuals alone.

The right order is:

1. `panel`
2. `dock`
3. `workspace`
4. `module`
5. `actions`
6. `welcome`
7. `app_ui`
8. `app`
9. `menu`, `commands`, `command_palette`, `status_bar`, `notifications`

Reason:

- runtime contracts should exist before shell composition
- otherwise the codebase will recreate another fake shell and patch behavior in later

## Decision Matrix

### Option A: Fork Zed and Delete Features

Pros:

- fast way to boot something that looks Zed-like
- immediate access to many existing shell/product pieces

Cons:

- wrong architecture center for a reusable host framework
- high risk of hidden coupling
- difficult to know what is safe to remove
- imports long-term product baggage
- likely to fight the current crate structure

When this option would make sense:

- the goal is a Zed-like product
- the repository accepts Zed-shaped architecture
- future features stay close to editor workflows

This is **not** the current situation.

### Option B: Keep This Repository and Rewrite the Kernel

Pros:

- architecture stays aligned with this repository's goals
- crate boundaries remain stable
- easier to keep `db` and runtime rules consistent with repository constraints
- future business modules attach to a framework the repository owns

Cons:

- slower first visual result
- requires discipline in runtime design

When this option makes sense:

- the goal is a framework kernel
- long-term extensibility matters more than a quick visual clone

This is the recommended option.

## Practical Rule of Thumb

If a Zed code path mainly solves:

- window chrome
- title bar composition
- shell menu mounting
- platform window controls

it is worth studying.

If a Zed code path mainly solves:

- editor tabs
- item lifecycle
- pane splits
- project/worktree state
- collaborative state
- language/runtime tooling

it should be treated as out of scope noise for this repository.

## Guardrails

Do not do the following:

- do not fork Zed and gradually delete toward a framework
- do not import Zed workspace concepts that this repository has not chosen explicitly
- do not let `ui` become another fake behavior layer
- do not let business modules invent private layout managers
- do not bypass the repository's crate boundaries just because Zed groups things differently

## Exit Criteria for This Decision

This strategy is being followed correctly when:

- the shell uses Zed as a visual and structural reference, not as a codebase dependency
- `workspace`, `dock`, and `panel` are implemented around this repository's own contracts
- menu, keymap, and command palette trigger typed actions
- `welcome` validates a real open/focus/toggle/restore lifecycle
- future modules can attach without editing core shell code

## Final Recommendation

Rebuild the workbench kernel in this repository.

Use Zed to inform:

- shell composition
- title bar structure
- menu placement
- native window control behavior
- panel lifecycle direction

Do not use Zed as the source tree to trim down.

The right target is not "a smaller Zed".

The right target is "this repository's own workbench framework, with Zed as a reference for the parts Zed is genuinely good at."
