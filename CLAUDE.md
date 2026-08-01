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
cargo test                        # everything (unit + integration + doc)
cargo test --lib                  # unit tests only (what CI measures coverage on)
cargo test --test '*'             # integration tests only (tests/*.rs)
cargo test --doc                  # doc tests only — `--all-targets` skips these
cargo test test_debouncer_batches_events   # single test by name
cargo test --lib watch::filter    # all tests in one module
cargo bench                       # criterion benches (benches/cli.rs)
cargo llvm-cov --lib              # coverage; needs `cargo install cargo-llvm-cov`
```

Gates that CI enforces (all must pass):

```shell
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings   # warnings are errors
cargo test --locked --all-targets                    # ubuntu-latest + macos-14
cargo test --locked --doc
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --locked
cargo +1.85.0 check --locked --all-targets           # MSRV, matches Cargo.toml rust-version
editorconfig-checker                                 # file formatting
```

CI also runs `typos` (crate-ci/typos) and `lychee` link-checking over `*.md`.

**`--all-targets` does not run doc tests** — the crate-level examples in `lib.rs` and `watcher.rs` only
compile under a separate `cargo test --doc`, which is why CI has both steps.

MSRV is **1.85.0**, not the 1.81.0 the crate used to claim: the pinned `globset` requires edition 2024.
Bumping `rust-version` in `Cargo.toml` means bumping the pinned toolchain in the `msrv` CI job too.

Docs site lives in `web/` (Astro + Starlight, separate npm project): `cd web && npm run dev`.

Releases are tag-triggered (`.github/workflows/cd.yml`) — see `RELEASE.md`; CHANGELOG is generated with
`git-cliff` (`cliff.toml`).

## Architecture

### Public library API (`src/lib.rs`, `src/watcher.rs`, `src/error.rs`)

The supported surface is what `lib.rs` re-exports: `Watcher`, `WatcherBuilder`, `Workspace`, `Config`,
`CommandType`, `DebouncerConfig`, `Error`, `Result`, `DEFAULT_IGNORE_PATTERNS`,
`default_ignore_patterns`. The modules stay `pub` because the binary is a separate crate, but they carry
weaker guarantees — prefer the root re-exports when adding examples or docs.

`Watcher::{builder, from_config, from_config_file}` all validate up front (non-empty workspaces,
non-blank `dir`, non-empty `cmd`, ignore patterns compile as globs), so `Watcher::run` only fails at
runtime. `run` spawns one task per workspace and uses `select_all`: the first failure aborts the
siblings and is returned as `Error::Workspace { dir, source }`. **Nothing in the library calls
`process::exit`** — only `main.rs` does, on the returned `Err`.

Every fallible entry point returns `crate::Result<T>` over the `#[non_exhaustive] Error` enum
(`ConfigRead`, `ConfigParse`, `InvalidConfig`, `Ignore`, `Watch`, `Workspace`, `Task`) — no
`Box<dyn Error>`, no bare `notify::Error`. `Workspace` has chainable setters (`cmd`, `cmds`, `bin_path`,
`bin_arg`, `ignore`, `extend_ignore`, `env_file`) plus the resolvers `ignore_patterns()` and
`env_vars()`; fields stay public so struct literals still work.

### Entry dispatch (`src/main.rs`)

Three mutually exclusive paths, decided in this order:

1. `init` subcommand → `init::run` (interactive scaffolder).
2. `--monitor` flag → `monitor::run` (TUI), returns without watching.
3. Otherwise: if `args.config` (default `rustywatch.yaml`, override `--cfg`) exists on disk →
   `run::config`, else → `run::cli` using flags only. The config file silently wins over CLI args.
   Both return `rustywatch::Result<()>`; `main` is the only place that maps `Err` to `exit(1)`.
   `run::cli` with no `--cmd` now errors instead of watching with an empty command.

### Watch pipeline (`src/watch/`)

`Watcher::run` spawns one `tokio::spawn` task per workspace (see above for its failure semantics). Each
task runs `watch::notify::watch(workspace, debounce)`, which chains:

```
notify::recommended_watcher (std::sync::mpsc callback)
  → std::thread bridge: keeps only EventKind::Modify(ModifyKind::Data(_)),
    drops paths matching CompiledFilter, forwards to a tokio mpsc
  → Debouncer (300ms quiet period, 2s hard max; override via WatcherBuilder)
    — dedupes paths into one batch
  → ReloadController::request_reload()
```

Only *data modification* events trigger reloads — file creation, deletion, and rename do not.

`ReloadController` (`reload_controller.rs`) is a 3-state coalescer: `Idle → Reloading → PendingReload`.
At most one reload runs and at most one is queued; bursts collapse into a single follow-up. The actual
work happens in `watch::reload::reload(&mut running_binary, &workspace, &env_vars)`, which kills the
previous child, deletes `bin_path`, runs the command(s), then respawns the binary.

### Path and environment resolution (easy to get wrong)

- **Working directory**: every command and binary spawn uses `.current_dir(work_dir)` where `work_dir`
  is the workspace `dir`. Configs must **not** prefix commands with `cd`.
- **`bin_path`**: absolute paths are used as-is; relative paths resolve to `cwd + dir + bin_path`
  (see `reload.rs`), i.e. relative to the workspace, not to the process CWD.
- **`env_file`**: a leading `/` means "relative to the project root" (the slash is stripped, it is *not*
  a filesystem-absolute path); anything else is relative to the workspace `dir`
  (`Workspace::env_vars`).

### Config (`src/config/schema.rs`)

`Config { workspaces: Vec<Workspace> }`. `Config::{from_file, from_yaml, to_yaml}` are the loaders (the
old `config::helper::read` is gone); `validate()` checks non-emptiness plus every workspace via
`Workspace::validate`. `cmd` uses a custom `deserialize_cmd` accepting either a YAML scalar or a
sequence → `CommandType::Single | Multiple`. **`CommandType::Multiple` runs all commands in parallel via
`join_all`, not sequentially** — the list syntax reads like steps but isn't ordered. `None` optional
fields are skipped on serialize, so `rustywatch init` emits no `null`s.

### Ignore patterns (`src/watch/ignore_defaults.rs`, `filter.rs`)

`merge_with_defaults` does **not** merge: a non-empty user `ignore` list *replaces*
`DEFAULT_IGNORE_PATTERNS` entirely. Users who set `ignore: ['.git']` lose `target/`, `node_modules/`,
etc. `Workspace::extend_ignore` is the library-side escape hatch — it seeds from the defaults first.

`filter.rs` has one matcher: `CompiledFilter` (globset-backed, handles `*.log`, `dir/`, dot-prefixed
patterns). It is compiled during validation, so an invalid glob surfaces as `Error::Ignore` at
`build()`/`from_config()` time rather than silently matching nothing. The legacy suffix/substring
`is_ignored` free function has been removed.

### Monitor (`src/monitor/`)

`app.rs` holds `App` state + the crossterm key loop; `ui.rs` renders; `theme.rs` styles; `service.rs`
(`ServiceTracker`) reads `rustywatch.yaml` and matches running processes against workspace commands and
`bin_path`s via `HashSet` lookups, with a periodic full rescan (every 10 refreshes). Keys: `q` quit,
`j/k` navigate, `1`–`5` sort column, `s`/`S` cycle/reverse sort, `/` search, `x` kill, `R` restart, `?`
help.

### Init (`src/init/`)

`detect.rs` sniffs project type by marker file (Cargo.toml → Rust, go.mod → Go, bun.lockb → Bun,
package.json → Node.js, pyproject/setup.py/requirements.txt → Python), extracts a project name, and
`scan_projects(root, max_depth)` walks the tree (default depth 2) returning a `DetectedProject` per
marker — skipping hidden dirs and `SKIP_DIRS` (target, node_modules, vendor, …) so vendored copies
aren't picked up. `detect_js_command` reads `package.json` scripts (`dev`>`start`>`serve`>`watch`) and
`detect_python_entry` picks an entry file that exists.

`templates.rs` maps each `ProjectType` to default cmd/bin_path/ignore; `ProjectTemplate::to_workspace
(config_dir, fs_dir, name)` takes the config-relative dir separately from the on-disk dir it probes.
**The Rust template watches the crate root, not `src/`** — a relative `bin_path` resolves against the
workspace `dir`, so `dir: src` would look for `src/target/debug/<name>` and silently never start the
binary.

`prompts.rs` drives the `inquire` flow: MultiSelect over detected projects → optional per-workspace
customization → validated preview. Pure helpers (`config_dir_for`, `workspace_from_detected`,
`default_workspaces`, `confirm_output_path`, `parse_patterns`, `parse_args`) are split out from the I/O
so they're unit-testable. `--yes` never prompts (existing output file → error asking for `--force`), and
a non-TTY without `--yes` errors early instead of hanging. `Config::validate` runs before anything is
written. `generator.rs` serializes to YAML with a header comment.

## Testing conventions

- Unit tests are `#[cfg(test)] mod tests` inline in each source file. `tests/` holds integration tests:
  `cli.rs` and `integration.rs` shell out to `cargo run`; `init.rs` and `library_api.rs` are newer and
  use `env!("CARGO_BIN_EXE_rustywatch")` instead — prefer that, it skips a nested cargo invocation and
  the build-lock contention that comes with it.
- **Never call `process::exit` on a code path reachable from tests** — it terminates the whole test
  binary and silently skips pending tests. Existing guards follow this: `watch::notify::watch` returns
  early under `cfg!(test)` instead of entering its event loop, `reload` returns before respawning a
  binary, and `run::cli` returns its `Result` to `main` rather than exiting itself.
- **`cfg!(test)` is false in integration tests** — they link the normal build. `Watcher::run` therefore
  watches forever from `tests/`; only construct and validate there, never run.
- The monitor is tested without touching the real system: `App::with_processes` (test-only) injects a
  fixed process list and an empty `System`, so sorting/navigation assertions are deterministic and
  `kill_process` can never signal a live PID. Do not call `refresh`/`update_process_list` on such an
  `App` — they replace the injected list.
- `ui.rs` renders into `ratatui::backend::TestBackend` and asserts on the flattened buffer text; the
  `render` helper covers all three responsive layouts by terminal size.
- Filesystem tests use `tempfile`; canonicalize temp paths before comparing (macOS `/var` → `/private/var`).

## Self-hosting

The repo's own `rustywatch.yaml` watches `web/` and every `examples/*` project — running `cargo run` at
the root starts the docs site plus the Go/Node/Rust/Bun example servers, which is the fastest end-to-end
check of watcher behavior.
