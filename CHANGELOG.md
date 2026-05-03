# Changelog

## Unreleased

- Added Japanese `README.md` with install, keybindings, mouse, and session file location.

## [0.1.0] - 2026-05-04

- Ratatui terminal multiplexer MVP with tmux-style Ctrl-b prefix controls.
- PTY-backed shell sessions per pane; projects, window pages, nested vertical/horizontal splits; TOML session persistence (cwd, names, layout, active selections).
- Sidebar project list with bold `kandume` title and `CARGO_PKG_VERSION`; vt100 cell rendering for ANSI colors.
- Mouse capture with sidebar project selection and XTerm SGR forwarding to panes (wheel, click, drag, release); hover motion disabled (`?1003l`) so shells do not print CSI fragments.
- Shared `layout` module used by both rendering and mouse hit-testing.
- New project (`Ctrl-b t`) opens shells under the user home directory (`dirs::home_dir()`).
- Pane titles, rename flows, close-last-pane guard, thin split separators, terminal status accents (running / active / completed / failed).
