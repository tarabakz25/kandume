# Contributing to kandume

## Prerequisites

- Rust stable (latest stable that supports the 2024 edition is recommended)
- Unix-like OS (macOS / Linux) — Windows is not supported because PTY is required

## Development setup

```sh
git clone https://github.com/tarabakz25/kandume.git
cd kandume
cargo build       # verify the build
cargo test        # run tests
cargo clippy      # lint
cargo fmt         # format
```

## Branch strategy

| Branch   | Purpose |
|----------|---------|
| `main`   | Stable released code. Direct pushes are forbidden. |
| `develop`| Integration branch. Homebrew `--HEAD` points here. |
| `feat/*` | New features |
| `fix/*`  | Bug fixes |
| `chore/*`| Build, CI, docs, and other housekeeping |

Branch off `develop`, then open a PR back to `develop`. The `develop → main` merge happens when cutting a release tag.

## Commit messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <summary>
```

- **type**: `feat` / `fix` / `refactor` / `test` / `docs` / `chore`
- **scope**: optional (e.g. `input`, `layout`, `session`, `ui`)
- **summary**: imperative mood, English

Examples:
```
feat(input): add Ctrl-b z to zoom active pane
fix(session): migrate v1 schema when project list is empty
```

## Pull requests

1. Rebase on the latest `develop` before opening a PR
2. All CI checks must pass (`fmt --check` / `clippy` / `test`)
3. Add an entry to the `## Unreleased` section of `CHANGELOG.md`
4. Describe **what** changed and **why** in the PR body
5. If PTY behaviour changed, include a brief note on manual verification

## Testing

- Logic tests live in `src/app.rs` under `#[cfg(test)]`
- PTY / TUI integration tests are not set up; manual testing is expected for those paths
- `cargo test` must pass as a minimum gate

## Coding conventions

- Use `pub(crate)` for intra-crate items; `pub` only when genuinely public API is needed
- Keep `cargo clippy` warning-free
- Follow `cargo fmt` defaults (no `rustfmt.toml`)
- Any code path that could panic must leave `install_panic_hook()` in place so the terminal is restored before the panic message is printed

## Session schema changes

`src/session.rs` holds three schema generations under `#[serde(untagged)]`. When modifying the schema, verify that all existing variants still deserialize correctly and add a new variant if needed. Do not change the deserialization order of existing variants.

## Release process (maintainers)

1. Replace `## Unreleased` in `CHANGELOG.md` with the version number and date
2. Bump `version` in `Cargo.toml`
3. Confirm `cargo build --release` succeeds
4. Merge `develop → main` via PR
5. Tag `main` with `git tag v<version>` and push the tag
6. Create a GitHub Release linked to the tag
7. Open a PR to update the Formula tarball URL and `sha256`
