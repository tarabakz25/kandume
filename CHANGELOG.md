# Changelog

## Unreleased

- chore(brew): point stable `Formula/kandume.rb` tarball at v0.1.1 with checksum.
- Close pane (`Ctrl-b x`): closing the last pane removes the session and focuses the previous session; if it was the only session, the current project slot is reset to a new home-directory project with one pane.
- UI copy: call the middle tier **session** (not window/tab); sidebar lists session count per project.

## [0.1.1] - 2026-05-04

- Homebrew: add `Formula/kandume.rb` so the repo can be used as `brew tap tarabakz25/kandume https://github.com/tarabakz25/kandume.git` and installed with `brew install kandume` (stable tarball) or `brew install kandume --HEAD` (branch `develop`).
- CLI: support `--version` / `-V` before starting the TUI (prints `CARGO_PKG_VERSION`).
- Japanese `README.md` with install, keybindings, mouse, and session file location.

## [0.1.0] - 2026-05-04

- Ratatui terminal multiplexer MVP with tmux-style Ctrl-b prefix controls.
- PTY-backed shell sessions per pane; projects, window pages, nested vertical/horizontal splits; TOML session persistence (cwd, names, layout, active selections).
- Sidebar project list with bold `kandume` title and `CARGO_PKG_VERSION`; vt100 cell rendering for ANSI colors.
- Mouse capture with sidebar project selection and XTerm SGR forwarding to panes (wheel, click, drag, release); hover motion disabled (`?1003l`) so shells do not print CSI fragments.
- Shared `layout` module used by both rendering and mouse hit-testing.
- New project (`Ctrl-b t`) opens shells under the user home directory (`dirs::home_dir()`).
- Pane titles, rename flows, close-last-pane guard, thin split separators, terminal status accents (running / active / completed / failed).
