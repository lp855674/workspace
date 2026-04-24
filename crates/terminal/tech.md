# Terminal

## Goal

This crate owns the local shell runtime behind the workbench bottom dock. It is the backend boundary between shell process management and UI rendering.

## Responsibilities

- spawn the platform local shell in the workspace cwd
- manage child process lifetime and best-effort shutdown
- maintain a PTY-backed `vt100` parser with bounded scrollback
- write user input bytes into the shell PTY
- expose a render-ready snapshot for `app_ui` / `ui`, including terminal cells, ANSI styles, cursor state, and scrollback position
- allow the workbench runtime to move the terminal viewport through scrollback without faking transcript state

## Non-Goals

- custom shell process orchestration outside the local workspace shell
- final terminal-view polish such as selection painting, search, links, or split panes
- dock registration or panel lifecycle

Those remain future work if the project adopts a fuller Zed-style `terminal + terminal_view` split.
