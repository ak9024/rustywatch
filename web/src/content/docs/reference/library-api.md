---
title: Library API
description: Use RustyWatch as a Rust crate instead of a CLI
---

RustyWatch is a library as well as a binary. Add it to a project and drive the
same watch/reload engine the CLI uses:

```shell
cargo add rustywatch
```

## Quick start

```rust
use rustywatch::{Watcher, Workspace};

#[tokio::main]
async fn main() -> rustywatch::Result<()> {
    Watcher::builder()
        .workspace(
            Workspace::new("./api")
                .cmd("cargo build")
                .bin_path("target/debug/api")
                .bin_arg(["--port", "8080"])
                .env_file(".env"),
        )
        .workspace(Workspace::new("./web").cmd("npm run dev"))
        .build()?
        .run()
        .await
}
```

`run()` drives every workspace concurrently and resolves only when one of them
fails. The first failure aborts the remaining workspaces and is returned as an
[`Error`](#errors) — nothing in the crate calls `process::exit`, so it is safe
to embed in a larger application.

## Reusing `rustywatch.yaml`

The same config file the CLI reads is available from code:

```rust
use rustywatch::{Config, Watcher};

async fn run_from_file() -> rustywatch::Result<()> {
    // Read, parse and validate in one step.
    Watcher::from_config_file("rustywatch.yaml")?.run().await
}

async fn inspect_then_run() -> rustywatch::Result<()> {
    // Or take the config apart first.
    let config = Config::from_file("rustywatch.yaml")?;
    println!("watching {} workspaces", config.workspaces.len());
    Watcher::from_config(config)?.run().await
}
```

`Config` also round-trips through YAML strings with `Config::from_yaml` and
`Config::to_yaml`.

## `Watcher`

| Method | Description |
|--------|-------------|
| `Watcher::builder()` | Start a [`WatcherBuilder`](#watcherbuilder). |
| `Watcher::from_config(config)` | Build from an already-parsed `Config`, validating it. |
| `Watcher::from_config_file(path)` | Read, parse and validate a config file. |
| `watcher.workspaces()` | The `&[Workspace]` that will be run. |
| `watcher.debounce()` | The `&DebouncerConfig` applied to every workspace. |
| `watcher.run().await` | Run every workspace until one fails. |

## `WatcherBuilder`

| Method | Description |
|--------|-------------|
| `.workspace(ws)` | Add one workspace. |
| `.workspaces(iter)` | Add several workspaces. |
| `.config(config)` | Add every workspace from a `Config`. |
| `.debounce_delay(Duration)` | Quiet period after the last file event (default 300ms). |
| `.max_debounce_delay(Duration)` | Hard ceiling on deferring a reload (default 2s). |
| `.build()` | Validate and produce a `Watcher`. |

Validation happens in `build()`, not at run time: an empty workspace list, a
blank `dir`, a missing command, or an ignore pattern that is not a valid glob
all fail before anything is spawned.

```rust
use std::time::Duration;
use rustywatch::{Watcher, Workspace};

fn build_watcher() -> rustywatch::Result<Watcher> {
    Watcher::builder()
        .workspace(Workspace::new(".").cmd("cargo test"))
        .debounce_delay(Duration::from_millis(50))
        .max_debounce_delay(Duration::from_millis(500))
        .build()
}
```

## `Workspace`

Every field of the YAML schema has a chainable setter. Fields stay public, so
struct literals keep working, but the builder survives new optional fields
being added.

| Method | Description |
|--------|-------------|
| `Workspace::new(dir)` | Watch `dir`; also the working directory for commands. |
| `.cmd(cmd)` | One command to run on change. |
| `.cmds(iter)` | Several commands — run **in parallel**, not in sequence. |
| `.bin_path(path)` | Binary to restart; relative paths resolve against `dir`. |
| `.bin_arg(iter)` | Arguments passed to that binary. |
| `.ignore(iter)` | **Replace** the default ignore patterns. |
| `.extend_ignore(iter)` | **Add to** the default ignore patterns. |
| `.env_file(path)` | `.env` file to load; a leading `/` means "project root". |

Two read-only helpers resolve what the watcher will actually use:

```rust
use rustywatch::Workspace;

let workspace = Workspace::new("./api").cmd("cargo build").env_file(".env");

// Defaults applied: target/, node_modules/, .git/, …
let patterns: Vec<String> = workspace.ignore_patterns();

// Parsed ./api/.env, or an empty map when the file is absent.
let env = workspace.env_vars();
```

### `ignore` replaces, `extend_ignore` adds

This is the single most common surprise. A non-empty `ignore` list *replaces*
the defaults, matching the YAML behaviour:

```rust
use rustywatch::Workspace;

// Loses target/, node_modules/, and every other default.
let strict = Workspace::new(".").cmd("cargo build").ignore([".git"]);
assert_eq!(strict.ignore_patterns(), vec![".git".to_string()]);

// Keeps the defaults and adds to them.
let relaxed = Workspace::new(".").cmd("cargo build").extend_ignore(["*.snap"]);
assert!(relaxed.ignore_patterns().contains(&"target/".to_string()));
```

The defaults themselves are exported as `rustywatch::DEFAULT_IGNORE_PATTERNS`
(a `&[&str]`) and `rustywatch::default_ignore_patterns()` (a `Vec<String>`).

## `CommandType`

`cmd` is either a single string or a list. The library accepts both, and
`CommandType` implements `From<&str>`, `From<String>` and `From<Vec<String>>`.

```rust
use rustywatch::CommandType;

let cmd = CommandType::from("cargo build");
assert_eq!(cmd.len(), 1);
assert_eq!(cmd.iter().collect::<Vec<_>>(), vec!["cargo build"]);
assert!(!cmd.is_empty());
```

The list form runs **in parallel**. When order matters, chain inside a single
command: `.cmd("npm install && npm start")`.

## Errors

Every fallible entry point returns `rustywatch::Result<T>`, whose error type is
the `rustywatch::Error` enum — no boxed trait objects.

| Variant | Raised when |
|---------|-------------|
| `ConfigRead { path, source }` | The config file could not be read. |
| `ConfigParse { path, source }` | The config file is not valid RustyWatch YAML. |
| `InvalidConfig(String)` | No workspaces, blank `dir`, or no command. |
| `Ignore { pattern, source }` | An ignore pattern is not a valid glob. |
| `Watch(notify::Error)` | The platform watcher could not start or observe a path. |
| `Workspace { dir, source }` | A workspace failed; `source` holds the cause. |
| `Task(JoinError)` | A workspace task panicked or was cancelled. |

The enum is `#[non_exhaustive]`, so match with a trailing `_ =>` arm:

```rust
use rustywatch::{Error, Watcher, Workspace};

fn load() -> rustywatch::Result<Watcher> {
    match Watcher::from_config_file("rustywatch.yaml") {
        Ok(watcher) => Ok(watcher),
        Err(Error::ConfigRead { path, .. }) => {
            eprintln!("no config at {path}, falling back to defaults");
            Watcher::builder()
                .workspace(Workspace::new(".").cmd("cargo run"))
                .build()
        }
        Err(e) => Err(e),
    }
}
```

`Error` implements `std::error::Error`, so `source()` gives the full chain and
`?` still converts into `Box<dyn Error>` or `anyhow::Error`.

## Behaviour worth knowing

- Only *data modification* events trigger reloads. Creating, deleting or
  renaming a file does not.
- Commands run with the workspace `dir` as their working directory — never
  prefix them with `cd`.
- A relative `bin_path` resolves against the workspace `dir`, not the process
  working directory.
- `env_file` treats a leading `/` as "relative to the project root", not as a
  filesystem-absolute path.
- A missing or unreadable `env_file` logs a warning and yields no variables
  rather than failing.

## Migrating from 0.2.x / 0.3.x

The previous library surface was a set of free functions taking six positional
arguments. It has been replaced:

| Before | Now |
|--------|-----|
| `run::run(dir, cmd, ignore, bin_path, bin_arg, env_vars)` | `Watcher::builder().workspace(...).build()?.run()` |
| `watch::notify::watcher(dir, cmd, ignore, bin_path, bin_arg, env_vars)` | `watch::notify::watch(workspace, debounce)` |
| `config::helper::read(path)` | `Config::from_file(path)` |
| `Config::validate() -> Result<(), String>` | `Config::validate() -> rustywatch::Result<()>` |
| `Result<(), Box<dyn Error>>` / `Result<(), notify::Error>` | `rustywatch::Result<T>` |
| `watch::filter::is_ignored(path, &patterns)` | `CompiledFilter::new(&patterns)?.is_ignored(path)` |
| `run::config` exiting the process on failure | `Watcher::run` returning `Err` |

The CLI, the `rustywatch.yaml` schema and the config file's precedence over CLI
flags are unchanged.
