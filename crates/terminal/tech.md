# Terminal

## Goal

This crate owns the local shell runtime behind the workbench bottom dock. It is the backend boundary between shell process management and UI rendering.

## Responsibilities

- spawn the platform local shell in the workspace cwd
- manage child process lifetime and best-effort shutdown
- maintain a PTY-backed `alacritty_terminal::Term` with native scrollback and display offset
- maintain terminal history bounds and viewport position as terminal-owned state
- write user input bytes into the shell PTY
- expose a viewport-aware snapshot for `app_ui` / `ui`, including terminal cells, ANSI styles, cursor state, retained history size, and viewport position
- allow the workbench runtime to move the terminal viewport through scrollback without adding a second UI scroll model
- expose the terminal's native display offset directly so UI scrolling matches Zed's history model

## Non-Goals

- custom shell process orchestration outside the local workspace shell
- final terminal-view polish such as selection painting, search, links, or split panes
- dock registration or panel lifecycle

Those remain future work if the project adopts a fuller Zed-style `terminal + terminal_view` split.

## Notes

- `TerminalSession` renders snapshots from `alacritty_terminal::Term::renderable_content()`.
- Scroll range now comes from `total_lines - screen_lines`, so resizing changes the viewport size without fabricating extra history.
- Input snaps the terminal back to `display_offset = 0` before writing to the PTY, matching Zed's follow-output behavior.
