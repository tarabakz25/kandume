# AGENTS.md

Rust 2024 edition TUI terminal multiplexer (PTY-backed, Ratatui). Single crate, no workspace.

## Commands

```sh
cargo build              # debug build
cargo build --release    # release build → target/release/kandume
cargo run                # run debug
cargo test               # all tests
cargo test <test_name>   # single test, e.g. cargo test split_leaf_nests_at_active_leaf
cargo clippy             # lint (no .clippy.toml; defaults apply)
cargo fmt                # format (no rustfmt.toml; defaults apply)
cargo fmt --check        # CI-style format check
```

No Makefile, Justfile, CI workflows, or task runner exists.

## Architecture

```
src/main.rs     — entrypoint, terminal init, panic hook (restores terminal on crash)
src/app.rs      — App struct, event loop, project/window/pane management; unit tests here
src/input.rs    — Ctrl-b prefix key handling
src/layout.rs   — pane layout calc (shared by render and mouse hit-test)
src/mouse.rs    — XTerm SGR mouse encoding
src/session.rs  — TOML session save/load, 3-generation schema migration
src/terminal.rs — PTY wrapper (TerminalTab via portable-pty)
src/ui.rs       — Ratatui draw calls
```

No codegen, no migrations to run manually (session schema migrates at runtime).

## Testing quirks

- Tests live in `src/app.rs` under `#[cfg(test)]` — only `PaneNode` layout logic is covered.
- No PTY/TUI integration tests; headless PTY testing is not set up.
- `cargo test` runs fast with no external services required.

## Runtime notes

- Requires Unix PTY (`portable-pty`). macOS and Linux only.
- `$SHELL` controls the shell spawned per pane; falls back to `/bin/zsh` if unset.
- Session file location:
  - macOS: `~/Library/Application Support/kandume/session.toml`
  - Linux: `~/.config/kandume/session.toml`
- `Ctrl-b d` saves session; `Ctrl-b q` quits **without** saving.

## Session schema

`session.rs` holds 3 schema generations under `#[serde(untagged)]`. Old files migrate automatically on startup — never edit schema deserialization without accounting for all three variants.

## Mouse handling quirk

`main.rs` manually sends `?1003l` (disable all-motion) immediately after `EnableMouseCapture`. Crossterm enables `?1003h` which floods PTYs with garbage on hover; the manual override is intentional — do not remove it.

## Panic hook

`install_panic_hook()` in `main.rs` restores the terminal before printing the panic message. Any code path that could crash must keep this in place.

## CHANGELOG.md

Update `CHANGELOG.md` for every code change (project convention from `~/.claude/CLAUDE.md`).

## Visibility convention

Use `pub(crate)` for intra-crate items; bare `pub` is not used in this codebase.
