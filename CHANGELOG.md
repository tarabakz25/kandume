# Changelog

## Unreleased

- Added a ratatui-based terminal multiplexer MVP with tmux-style prefix controls.
- Added per-tab PTY-backed shell sessions, tab switching, tab rename, and tab close behavior.
- Added TOML session persistence for tab names, working directories, shells, and active tab index.
- Changed the tab UI from a top bar to a left sidebar and resized PTY sessions to match the terminal pane.
- Changed the model so sidebar tabs represent projects, with each project containing multiple terminal sessions.
- Preserved terminal color output by rendering vt100 cell attributes while leaving default foreground/background colors to the user's terminal theme.
