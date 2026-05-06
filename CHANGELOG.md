# Changelog

## [Unreleased]

- feat(ui): highlight active pane with a cyan border overlay (`Block::border_style`) to visually distinguish the focused pane from inactive ones. Closes #9.
- feat(ui): display `NORMAL` mode indicator (green badge) in footer when no prefix is active; keybinding hints are shown only after `Ctrl-b` prefix, matching tmux-style mode display. Closes #10.
- feat(ui): add per-project status indicator in sidebar (`●` running/active, `○` all completed, `!` any failed) to surface PTY health at a glance. Closes #13.
- feat(ui): render `┼` (cross) box-drawing characters at separator intersections, fixing incorrect `│`/`─` overwrites when panes are split both vertically and horizontally. Closes #14.
- feat(session): add `Ctrl-b D` keybinding to delete the active project with a footer confirmation prompt; immediately persists the updated session to remove the entry from `session.toml`. Closes #8.
- feat(app): add `Ctrl-b e` to edit the active pane's shell command; the current shell path is pre-filled in the footer editor and on Enter the PTY is killed and restarted with the new command. Closes #12.

## [0.1.2] - 2026-05-06

- fix(mouse): do not forward SGR mouse bytes to PTY unless the child app has enabled a mouse protocol mode (`?1000h` / `?1002h` / `?1003h`). Clicks on a plain shell previously printed `0;col;rowM3;col;rowm` garbage; now only apps that requested mouse tracking receive the encoded events. The same guard applies to drag, release, and scroll events.
- feat(mouse): pane-title click — clicking a pane's title row activates that pane.
- feat(mouse): separator drag-resize — dragging a vertical/horizontal split separator resizes both halves in real time; split ratio (`f64`) persisted in session TOML.
- feat(mouse): separator hover highlight — hovering over a separator turns it yellow (`Color::Yellow`); `?1003h` all-motion tracking is now active (removed the previous `?1003l` override in `main.rs`).
- refactor(layout): introduce `split_chunks`, `hit_separator`, `SeparatorHit`, `split_first_size`; all layout calculations use ratio instead of fixed 50/50 split.
- refactor(app): `PaneNode::Split` gains `ratio: f64`; `WhichChild`, `SplitPath`, `SeparatorDrag` types added; `ratio_at_path_mut` for in-place drag updates.
- refactor(session): `SessionPaneLayout::Split` gains `ratio: f64` with `#[serde(default)]`; old session files migrate automatically to `0.5`.
- refactor(ui): `draw_pane_node` accepts `hover_sep: Option<&[WhichChild]>` and uses `layout::split_chunks` for consistent rendering; separator color driven by highlight state.

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
