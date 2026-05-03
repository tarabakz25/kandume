# Changelog

## Unreleased

- Enabled crossterm mouse capture on startup (SGR / xterm-style tracking); restored terminal mouse mode on exit and panic.
- Added sidebar mouse clicks to select projects using the same layout as the project list widget.
- Replaced the sidebar ASCII banner with a bold `kandume` title and `CARGO_PKG_VERSION` line above the project list.
- Forward mouse wheel, click, drag, and release sequences to the PTY using XTerm SGR mouse reporting so terminal apps (for example vim with `:set mouse=a`) receive mouse input.
- Disabled hover-only motion tracking (`?1003l` after capture) and no longer forward `MouseMoved` to PTYs so interactive shells do not echo CSI fragments as stray characters.
- Changed Ctrl-b `t` (new project) to spawn shells under the user home directory (`dirs::home_dir()`).
- Added shared `layout` helpers used by both rendering and hit-testing so pane rectangles stay aligned with the UI.

- Added a ratatui-based terminal multiplexer MVP with tmux-style prefix controls.
- Added per-tab PTY-backed shell sessions, tab switching, tab rename, and tab close behavior.
- Added TOML session persistence for tab names, working directories, shells, and active tab index.
- Changed the tab UI from a top bar to a left sidebar and resized PTY sessions to match the terminal pane.
- Changed the model so sidebar tabs represent projects, with each project containing multiple terminal sessions.
- Preserved terminal color output by rendering vt100 cell attributes while leaving default foreground/background colors to the user's terminal theme.
- Added project-scoped window pages and split terminal panes inside each window.
- Changed pane rendering from boxed panels to thin split separators and added nested vertical/horizontal split layouts.
- Changed the sidebar to show project session counts without window/pane split counts.
- Added terminal status accents for active output, completed shells, and failed shells.
