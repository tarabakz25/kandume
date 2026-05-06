# kandume

A terminal multiplexer built with [Ratatui](https://github.com/ratatui-org/ratatui). Organizes work into a **project → session → pane** hierarchy and uses a [tmux](https://github.com/tmux/tmux)-style **Ctrl-b prefix** for keybindings. Each pane runs a real shell backed by a PTY.


https://github.com/user-attachments/assets/5b7d5089-4428-4345-95ed-b54d2be4c819


## Requirements

- Rust toolchain (built with the 2024 edition)
- Unix-like OS with PTY support — macOS and Linux only
- `$SHELL` is used as the shell per pane; falls back to `/bin/zsh` if unset

## Installation

### Homebrew (macOS / Linuxbrew)

Register this repository as a [Homebrew tap](https://docs.brew.sh/Taps) and install:

```sh
brew tap tarabakz25/kandume https://github.com/tarabakz25/kandume.git
brew install kandume
```

To install the latest commit from the development branch:

```sh
brew install kandume --HEAD
```

`--HEAD` builds from the `develop` branch of the tap.

To update the Formula tarball to a new tag after a release, get the `sha256` and update `Formula/kandume.rb`:

```sh
curl -sL "https://github.com/tarabakz25/kandume/archive/refs/tags/v0.1.2.tar.gz" | shasum -a 256
```

(Replace `v0.1.2` with the actual tag name.)

### From source

```sh
git clone https://github.com/tarabakz25/kandume.git
cd kandume
cargo build --release
```

The binary is written to `target/release/kandume`.

## Usage

```sh
./target/release/kandume
```

On startup, the left side shows the project list and the right side shows the sessions and panes of the active project. Keystrokes are forwarded to the active pane's shell; all other input is handled by the UI or the prefix layer.

### Ctrl-b prefix

Press **Ctrl-b** once to enter prefix mode, then press a second key to run a command. A hint is shown in the status line.

| Key | Action |
|-----|--------|
| **Ctrl-b** then **Ctrl-b** | Send literal `^B` (0x02) to the shell |
| **t** | New project (working directory: home directory) |
| **c** | Add a session to the current project |
| **%** | Split the current session vertically |
| **"** | Split the current session horizontally |
| **n** / **p** | Next / previous project |
| **1–9** | Select a project by number |
| **]** / **[** | Next / previous session |
| **o** / **;** | Next / previous pane |
| **x** | Close the active pane. If it is the only pane in the session, the session is also closed and focus moves to the previous session. If the project has no sessions left, the slot is replaced with a new home-directory project (1 session, 1 pane). |
| **,** / **.** / **r** | Rename the current project / session / pane |
| **d** | Save session and quit |
| **q** | Quit without saving |
| **?** | Toggle help overlay |

Press **Enter** to confirm a rename, **Esc** to cancel.

### Mouse

In terminals that support mouse input, clicking a project row in the left sidebar switches to that project. Clicks, drags, and scroll wheel events over a pane are forwarded to the PTY in XTerm SGR format (e.g. Vim: `:set mouse=a`). Hover-only motion is suppressed so shells do not receive spurious CSI sequences.

### Session persistence

When you exit with **Ctrl-b d**, the layout, names, and working directories are saved as TOML.

- **Path**: `kandume/session.toml` inside the platform config directory
  - macOS: `~/Library/Application Support/kandume/session.toml`
  - Linux: `~/.config/kandume/session.toml`
- **Ctrl-b q** exits without saving.

## Development

```sh
cargo build
cargo test
cargo run
```
