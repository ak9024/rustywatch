# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

RustyWatch is a single-binary CLI (crate `rustywatch`, MSRV 1.81.0) that provides live reloading for any
language. It watches one or more workspace directories, re-runs build/run commands on change, and
optionally restarts a compiled binary. Also ships a `--monitor` ratatui TUI dashboard.

`src/lib.rs` is the library root and `src/main.rs` the thin binary — integration tests and benches drive
the CLI through `cargo run`.

## Commands

```shell
cargo build                       # debug build
cargo test                        # everything (unit + integration)
cargo test --lib                  # unit tests only (what CI measures coverage on)
cargo test --test '*'             # integration tests only (tests/cli.rs, tests/integration.rs)
cargo test test_debouncer_batches_events   # single test by name
cargo test --lib watch::filter    # all tests in one module
cargo bench                       # criterion benches (benches/cli.rs)
cargo llvm-cov                    # coverage; needs `cargo install cargo-llvm-cov`
```

Lint gates that CI enforces (all must pass):

```shell
cargo fmt --all -- --check
cargo clippy -- -D warnings       # warnings are errors
editorconfig-checker              # file formatting
```

CI also runs `typos` (crate-ci/typos) and `lychee` link-checking over `*.md`.

Docs site lives in `web/` (Astro + Starlight, separate npm project): `cd web && npm run dev`.

Releases are tag-triggered (`.github/workflows/cd.yml`) — see `RELEASE.md`; CHANGELOG is generated with
`git-cliff` (`cliff.toml`).

## Architecture

### Entry dispatch (`src/main.rs`)

Three mutually exclusive paths, decided in this order:

1. `init` subcommand → `init::run` (interactive scaffolder).
2. `--monitor` flag → `monitor::run` (TUI), returns without watching.
3. Otherwise: if `args.config` (default `rustywatch.yaml`, override `--cfg`) exists on disk →
   `run::config`, else → `run::cli` using flags only. The config file silently wins over CLI args.

### Watch pipeline (`src/watch/`)

Per workspace, `run::config` spawns one `tokio::spawn` task; `join_all` collects them and **any failed
task calls `process::exit(1)`**, killing all other workspaces. Each task runs `watch::notify::watcher`,
which chains:

```
notify::recommended_watcher (std::sync::mpsc callback)
  → std::thread bridge: keeps only EventKind::Modify(ModifyKind::Data(_)),
    drops paths matching ignore patterns, forwards to a tokio mpsc
  → Debouncer (300ms quiet period, 2s hard max) — dedupes paths into one batch
  → ReloadController::request_reload()
```

Only *data modification* events trigger reloads — file creation, deletion, and rename do not.

`ReloadController` (`reload_controller.rs`) is a 3-state coalescer: `Idle → Reloading → PendingReload`.
At most one reload runs and at most one is queued; bursts collapse into a single follow-up. The actual
work happens in `watch::reload::reload`, which kills the previous child, deletes `bin_path`, runs the
command(s), then respawns the binary.

### Path and environment resolution (easy to get wrong)

- **Working directory**: every command and binary spawn uses `.current_dir(work_dir)` where `work_dir`
  is the workspace `dir`. Configs must **not** prefix commands with `cd`.
- **`bin_path`**: absolute paths are used as-is; relative paths resolve to `cwd + dir + bin_path`
  (see `reload.rs`), i.e. relative to the workspace, not to the process CWD.
- **`env_file`**: a leading `/` means "relative to the project root" (the slash is stripped, it is *not*
  a filesystem-absolute path); anything else is relative to the workspace `dir` (`run.rs`).

### Config (`src/config/schema.rs`)

`Config { workspaces: Vec<Workspace> }`, validated only for non-emptiness. `cmd` uses a custom
`deserialize_cmd` accepting either a YAML scalar or a sequence → `CommandType::Single | Multiple`.
**`CommandType::Multiple` runs all commands in parallel via `join_all`, not sequentially** — the list
syntax reads like steps but isn't ordered.

### Ignore patterns (`src/watch/ignore_defaults.rs`, `filter.rs`)

`merge_with_defaults` does **not** merge: a non-empty user `ignore` list *replaces*
`DEFAULT_IGNORE_PATTERNS` entirely. Users who set `ignore: ['.git']` lose `target/`, `node_modules/`,
etc.

`filter.rs` has two matchers. `CompiledFilter` (globset-backed, handles `*.log`, `dir/`, dot-prefixed
patterns) is fully implemented and tested but **not wired into the watch path**; the hot path still uses
the legacy `is_ignored` free function, which only does suffix/substring matching. Prefer wiring up
`CompiledFilter` over extending the legacy function.

### Monitor (`src/monitor/`)

`app.rs` holds `App` state + the crossterm key loop; `ui.rs` renders; `theme.rs` styles; `service.rs`
(`ServiceTracker`) reads `rustywatch.yaml` and matches running processes against workspace commands and
`bin_path`s via `HashSet` lookups, with a periodic full rescan (every 10 refreshes). Keys: `q` quit,
`j/k` navigate, `1`–`5` sort column, `s`/`S` cycle/reverse sort, `/` search, `x` kill, `R` restart, `?`
help.

### Init (`src/init/`)

`detect.rs` sniffs project type by marker file (Cargo.toml → Rust, go.mod → Go, bun.lockb → Bun,
package.json → Node.js, pyproject/setup.py/requirements.txt → Python) and extracts a project name;
`templates.rs` maps each `ProjectType` to default cmd/bin_path/ignore; `prompts.rs` drives the `inquire`
flow (multi-workspace) or the `--yes` single-workspace path; `generator.rs` serializes to YAML.

## Testing conventions

- Unit tests are `#[cfg(test)] mod tests` inline in each source file; `tests/` holds CLI-level tests that
  shell out to `cargo run`.
- **Never call `process::exit` on a code path reachable from tests** — it terminates the whole test
  binary and silently skips pending tests. Existing guards follow this: `watcher` returns early under
  `cfg!(test)` instead of entering its event loop, `reload` returns before respawning a binary, and
  `run::cli` returns its `Result` to `main` rather than exiting itself.
- Filesystem tests use `tempfile`; canonicalize temp paths before comparing (macOS `/var` → `/private/var`).

## Self-hosting

The repo's own `rustywatch.yaml` watches `web/` and every `examples/*` project — running `cargo run` at
the root starts the docs site plus the Go/Node/Rust/Bun example servers, which is the fastest end-to-end
check of watcher behavior.
