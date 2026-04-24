# Zed Terminal Viewport And Button Visibility Design

## Goal

Fix two user-visible workbench problems by aligning behavior more closely with Zed:

- icon buttons in the window title bar and terminal header are not visually clear
- terminal scrolling is truncated and cannot move through the full retained history

This design prioritizes interaction correctness over minimal patch size.

## Problem Summary

The current terminal implementation mixes two incompatible scrolling models:

- `terminal` exposes only the currently visible terminal rows from `vt100`
- `app_ui` stores a logical terminal scrollback offset
- `ui` wraps the terminal output inside a native scrollable container

That creates a double-scroll system:

- terminal state tries to move the viewport through history
- UI scrolling moves a clipped slice of already-rendered content

The result is that scrolling does not cover the full retained transcript and early content is effectively cut off.

The current button rendering has a separate clarity problem:

- title bar controls depend on low-contrast icon presentation
- terminal header actions render as small icon glyphs without a strong button boundary

Users cannot easily identify the actions at a glance.

## Current Root Causes

### Terminal Scrolling

Current behavior is constrained by two implementation choices:

1. `crates/app_ui/src/app_ui.rs` clamps terminal viewport scrollback to `max_safe_scrollback().min(visible_rows)`, which effectively limits upward movement to roughly one viewport height.
2. `crates/ui/src/ui.rs` renders terminal content inside `.overflow_y_scrollbar()`, which introduces a second scroll state on top of terminal scrollback.

These choices are incompatible with Zed-style terminal behavior, where the terminal viewport is the single source of truth and the scrollbar only reflects that viewport.

### Icon Buttons

Current icon presentation does not provide enough interaction affordance:

- insufficient contrast against the current theme in the title bar
- too little separation between icon glyph and clickable button shape
- no strong hover/active feedback for terminal header actions
- action semantics are not obvious without guessing from the icon shape

## Reference Direction From Zed

This workspace should follow Zed's interaction model rather than reproducing its entire terminal architecture:

- terminal history and viewport are one coherent state machine
- UI does not independently scroll a pre-clipped transcript slice
- scrollbars and wheel input manipulate the terminal viewport, not an outer container
- icon actions use explicit button chrome, hover feedback, and stable hit targets

This design does not require a full `terminal + terminal_view` split, but it should preserve a future path toward that architecture.

## Scope

### In Scope

- redesign terminal viewport ownership so there is a single scrolling source of truth
- expose enough terminal snapshot metadata for viewport-aware rendering
- remove native content scrolling from the terminal text surface
- improve title bar control visibility within the current `gpui_component::TitleBar` approach
- redesign terminal header actions to render as explicit icon buttons
- add tests for viewport behavior, button visibility state, and regression boundaries

### Out Of Scope

- text selection
- search
- link detection
- split panes
- minimap-like terminal overview
- full Zed terminal architecture adoption
- unrelated shell or dock refactors

## Design

### 1. Single-Source Terminal Viewport

`crates/terminal` becomes the owner of terminal history bounds and viewport slicing.

The terminal snapshot must provide enough metadata for UI to render and control a viewport without inventing its own scroll state. The snapshot should include at least:

- `total_lines`
- `viewport_top`
- `viewport_height`
- `can_scroll_up`
- `can_scroll_down`
- viewport `visible_cells`
- viewport `visible_lines`

`app_ui` may still remember per-tab viewport intent, but it must do so in terminal terms:

- scroll by N lines
- page up/down
- jump to bottom
- restore a prior viewport for a tab

`ui` must only render the current viewport returned by terminal state. It must not create a second scroll model around terminal output.

### 2. Terminal Scroll Behavior

The terminal interaction model should behave as follows:

- mouse wheel scroll adjusts the terminal viewport offset
- PageUp and PageDown adjust the terminal viewport offset
- jump-to-bottom resets the viewport to the live bottom
- typing into the terminal while scrolled up returns the viewport to the bottom before sending input
- resizing keeps the viewport offset within valid history bounds

Each terminal tab keeps its own viewport position. Switching tabs restores that tab's previous terminal viewport.

### 3. UI Rendering Changes

`crates/ui` terminal rendering should change in one important way:

- the terminal text surface must no longer use the normal vertical overflow container as its history mechanism

If a scrollbar remains visible, it is only a representation of terminal viewport state. It must not become an independent scroll source.

The terminal content region should remain a fixed viewport whose content is replaced by the terminal snapshot for the current offset.

### 4. Title Bar Control Visibility

The app should continue using `gpui_component::TitleBar` and real window control hit areas. The change is visual clarity, not a replacement of the platform-aware title bar integration.

Requirements:

- window controls remain real title bar controls, not fake buttons
- foreground contrast must make the controls readable in the default theme
- hover states must clearly separate normal hover from close-button hover
- the controls must remain visually discoverable on first glance

If `TitleBar` styling hooks are limited, the implementation may add surrounding styling only if it does not interfere with hit testing or drag behavior.

### 5. Terminal Header Actions

Terminal header actions should be rendered as explicit icon buttons with:

- stable clickable size
- visible background or boundary
- hover feedback
- active and disabled states where applicable
- tooltip text for discoverability

This applies to:

- new terminal
- close terminal
- jump to bottom

The jump-to-bottom action appears only when the terminal is away from the live bottom.

### 6. Component Responsibilities

#### `crates/terminal`

Own:

- PTY lifecycle
- parser state
- retained terminal history bounds
- viewport offset validation
- viewport snapshot generation

Expose:

- viewport-aware snapshot data
- viewport movement operations
- jump-to-bottom operation
- max legal viewport range after resize or refresh

#### `crates/app_ui`

Own:

- per-tab terminal runtime state
- mapping wheel and keyboard events into terminal viewport actions
- terminal tab activation and viewport restoration
- title bar composition and terminal header state wiring

Do not own:

- an independent visual scroll container for terminal transcript

#### `crates/ui`

Own:

- rendering the provided terminal viewport
- rendering explicit terminal action buttons
- rendering title bar and terminal control affordances clearly

Do not own:

- transcript history pagination logic
- a second terminal scroll model

## Acceptance Criteria

- terminal scroll can move from the newest output back to the earliest retained history, not just one viewport height
- mouse wheel, page navigation keys, and jump-to-bottom all manipulate the same terminal viewport state
- terminal output is not wrapped in a native vertical overflow container that acts as transcript history
- switching terminal tabs preserves each tab's viewport position
- typing while scrolled away from bottom returns the active terminal to the live bottom before sending input
- window title bar controls are visually clear in the default theme
- terminal header buttons are visually clear in the default theme
- jump-to-bottom is visible only when the terminal is away from bottom

## Testing

### Terminal Tests

- scrollback history larger than the viewport remains reachable
- jump-to-bottom clears away-from-bottom state
- viewport offset remains valid after resize
- viewport clamping uses total retained history, not viewport height

### App Frame Tests

- wheel and keyboard navigation dispatch terminal viewport movement only
- terminal tabs preserve viewport independently
- typing while away from bottom snaps back before input is written

### UI Tests

- terminal render surface does not rely on native overflow scrolling for transcript history
- terminal header actions render with stable button structure
- jump-to-bottom visibility follows away-from-bottom state
- title bar composition still uses the real title bar integration

## Risks

### `vt100` History Access

The main technical risk is whether the current `vt100` usage can directly support the desired viewport model. If it cannot expose enough retained-history information, the terminal crate may need a thin history abstraction instead of relying purely on the current visible-screen snapshot approach.

### Title Bar Styling Limits

`gpui_component::TitleBar` may not expose every styling hook needed for ideal control visibility. If so, the implementation must improve clarity without breaking:

- drag behavior
- double-click zoom behavior
- platform hit testing for min/max/close

### Resize Semantics

Changing terminal height while scrolled away from bottom can cause invalid or jumpy offsets if the viewport policy is not explicit. The implementation must define and test resize behavior carefully.

## Rollback

If the viewport redesign causes regressions:

- keep the terminal as the single logical scroll source
- temporarily reduce viewport features, but do not reintroduce native transcript scrolling as a second scroll model

If title bar styling changes regress behavior:

- retain `gpui_component::TitleBar`
- revert only visual styling layers that do not affect window-control hit areas

## Verification

Run:

- `cargo test -p terminal -p ui -p app_ui -- --nocapture`
- `cargo check -p terminal -p ui -p app_ui`
- `powershell -ExecutionPolicy Bypass -File .\\script\\verify.ps1`
