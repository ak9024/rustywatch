# RustyWatch

![Crates.io Total Downloads](https://img.shields.io/crates/d/rustywatch)
![Crates.io License](https://img.shields.io/crates/l/rustywatch)
![docs.rs](https://img.shields.io/docsrs/rustywatch?style=social) ![Crates.io Size](https://img.shields.io/crates/size/rustywatch?style=flat) ![GitHub Repo stars](https://img.shields.io/github/stars/ak9024/rustywatch)
![GitHub Tag](https://img.shields.io/github/v/tag/ak9024/rustywatch)
![Crates.io Version](https://img.shields.io/crates/v/rustywatch)
![Codecov](https://img.shields.io/codecov/c/github/ak9024/rustywatch)

![RustyWatch](rustywatch.png)

## Live Reloading Built with Rust

Inspired by [Go Air](https://github.com/air-verse/air), RustyWatch provides powerful live reloading capabilities designed for developers working across various programming languages.

## Features

- **Universal Live Reloading:** Supports live reloading for any programming language (Go, Rust, Node.js, Python, and more).
- **Real-time Binary Reloading:** Automatically rebuilds and restarts your binaries on file changes.
- **Monorepo & Multi-Project Support:** Run multiple projects concurrently with a single command, and scaffold them all in one `rustywatch init`.
- **Automatic Working Directory:** Commands execute in the workspace `dir` automatically - no `cd` prefix needed.
- **Process Monitoring Dashboard:** Built-in terminal UI (`--monitor`) with real-time CPU/memory tracking, process management, and system metrics.
- **Smart File Filtering:** Glob-based ignore patterns for common build artifacts (`.git`, `node_modules`, `target/`, etc.), compiled and validated up front.
- **Async & High Performance:** Non-blocking async I/O with Tokio, tunable event debouncing, and coalesced reloads.
- **Usable as a Library:** The same watch/reload engine ships as a crate with a typed `Watcher`/`WatcherBuilder` API — no `process::exit`, every failure is a typed `Error`.
- **Cross-Platform:** Works on macOS, Linux, and Windows.
- **Flexible Configuration:** YAML-based config or CLI arguments for quick usage.

## Install

Requires Rust **1.85.0** or newer (MSRV, enforced in CI).

### Using Cargo

> curl --proto '=https' --tlsv1.2 -sSf <https://sh.rustup.rs> | sh

```shell
cargo install rustywatch
```

Or straight from git:

```shell
cargo install --git https://github.com/ak9024/rustywatch rustywatch
```

### Using Homebrew

```shell
# Add the tap
brew tap ak9024/rustywatch
# Install rustywatch
brew install rustywatch
```

## Quick Start

### Initialize Configuration

Generate a configuration file interactively:

```shell
rustywatch init
```

Or use auto-detected defaults:

```shell
rustywatch init --yes
```

### Run RustyWatch

To start the project, ensure you have a `rustywatch.yaml` configuration file in the root directory of your project. Then, run the CLI from the root directory to launch RustyWatch.

## Configuration

The default configuration file is named `rustywatch.yaml`, and it must be located in your project's root directory. For a reference configuration, please see the example below:

```yaml
# define workspaces, rustywatch can be handled multi project at the same time.
# commands automatically run in the workspace directory - no cd needed!
workspaces:
  # first project binary apps
  - dir: 'golang-project' # define path directory
    cmd: 'go build main.go' # runs in golang-project/
    bin_path: './main' # relative to workspace dir (golang-project/)
    bin_arg: # define arguments
     - server
    ignore:
     - '.git'
    env_file: '.env' # load environment variables from golang-project/.env
  # second project binary apps
  - dir: 'rust-project'
    cmd: 'cargo build' # runs in rust-project/
    bin_path: './target/debug/rust-project' # relative to workspace dir
    env_file: '/.env' # load environment variables from project root .env
  # third project non binary apps
  - dir: 'nodejs-project'
    cmd: 'npm run dev' # runs in nodejs-project/
  # cmd also accepts a list — note: the entries run in PARALLEL, not in sequence
  - dir: 'python-project'
    cmd:
     - 'python -m mypkg'
     - 'python -m mypkg.worker'
  # more ...
```

### Things that surprise people

- **`ignore` replaces the defaults, it does not extend them.** Setting
  `ignore: ['.git']` drops `target/`, `node_modules/` and the rest of
  `DEFAULT_IGNORE_PATTERNS`. List everything you need, or use
  `Workspace::extend_ignore` from the library.
- **A `cmd` list runs in parallel.** It reads like ordered steps but isn't —
  chain with `&&` inside a single command when order matters.
- **No `cd` prefix.** Every command and binary spawn already runs with the
  workspace `dir` as its working directory.
- **Relative `bin_path` resolves against the workspace `dir`**, not the process
  CWD. Absolute paths are used as-is.
- **`env_file` with a leading `/` means "relative to the project root"** — the
  slash is stripped, it is not a filesystem-absolute path. Anything else is
  relative to the workspace `dir`.
- **Only data-modification events trigger a reload.** Creating, deleting or
  renaming a file does not.
- **The config file wins over CLI flags.** If `rustywatch.yaml` (or whatever
  `--cfg` points at) exists, the watch flags are ignored.

```shell
# list directories
ls 
.
└── your-project/
    ├── go-project/
    │   ├── go.mod
    │   ├── go.sum
    │   └── main.go
    ├── rust-project/
    │   ├── src/
    │   │   └── main.rs
    │   ├── Cargo.toml
    │   └── Cargo.lock
    ├── nodejs-project/
    │   ├── index.js
    │   ├── package.json
    │   └── package-lock.json
    └── rustywatch.yaml (config here)
```

### Run the project

```shell
rustywatch
```

## Commands

### Watch Options

Used when no `rustywatch.yaml` is present — a single workspace straight from
flags. `--cmd` is required in this mode.

```shell
rustywatch [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-d, --dir <DIR>` | Directory to watch (default: `.`) |
| `-c, --cmd <CMD>` | Command to run on change (repeatable; entries run in parallel) |
| `-i, --ignore <PATTERN>` | Ignore pattern (repeatable; **replaces** the defaults) |
| `--bin_path <PATH>` | Binary to restart, relative to `--dir` |
| `--bin_arg <ARG>` | Argument passed to the binary (repeatable) |
| `--cfg <FILE>` | Config file path (default: `rustywatch.yaml`) |
| `--monitor` | Show the process monitoring dashboard instead of watching |

### Init Command

Generate a configuration file interactively:

```shell
rustywatch init [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-o, --output <FILE>` | Config file path (default: rustywatch.yaml) |
| `-d, --dir <DIR>` | Root directory to scan (default: `.`) |
| `--depth <N>` | How deep to scan for nested projects (default: 2, 0 = root only) |
| `--yes` | Skip prompts, accept every detected project as-is |
| `-f, --force` | Overwrite existing config file |
| `--dry-run` | Print the configuration instead of writing it |

`init` scans subdirectories, so a monorepo is configured in one run — every
detected project becomes a workspace, complete with its build command, binary
path, ignore patterns and `.env` file. Auto-detects Rust, Go, Node.js, Bun and
Python, reading `package.json` scripts and Python entry points so the generated
command points at something that exists.

```shell
rustywatch init --yes --dry-run    # preview without writing
```

### Process Monitor

Launch with the built-in TUI dashboard for real-time monitoring:

```shell
rustywatch --monitor
```

It reads `rustywatch.yaml` and matches running processes against each
workspace's commands and `bin_path`, so you see only the processes RustyWatch
cares about, alongside system metrics.

| Key | Action |
|-----|--------|
| `q` | Quit |
| `j` / `k` | Navigate |
| `1`–`5` | Sort by column |
| `s` / `S` | Cycle / reverse sort |
| `/` | Search |
| `x` | Kill process |
| `R` | Restart process |
| `?` | Help |

## Use as a Rust library

RustyWatch is a crate as well as a binary — the same watch/reload engine is
available from code.

```shell
cargo add rustywatch
```

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

Already have a `rustywatch.yaml`? Reuse it:

```rust
rustywatch::Watcher::from_config_file("rustywatch.yaml")?.run().await
```

`run()` drives every workspace concurrently and returns the first failure as a
typed `rustywatch::Error` — nothing in the crate calls `process::exit`, so it is
safe to embed in a larger application. `builder()`, `from_config()` and
`from_config_file()` all validate up front (non-empty workspaces, non-blank
`dir`, non-empty `cmd`, ignore patterns compile as globs), so a bad config fails
at `build()` rather than mid-run.

Keeping the default ignore patterns while adding your own:

```rust
Workspace::new("./api").cmd("cargo build").extend_ignore(["*.snap"])
```

Events are debounced 300ms (2s hard cap under a continuous stream of writes).
Both are tunable:

```rust
use std::time::Duration;

let watcher = rustywatch::Watcher::builder()
    .workspace(rustywatch::Workspace::new(".").cmd("cargo test"))
    .debounce_delay(Duration::from_millis(50))
    .max_debounce_delay(Duration::from_millis(500))
    .build()?;
```

Supported surface: `Watcher`, `WatcherBuilder`, `Workspace`, `Config`,
`CommandType`, `DebouncerConfig`, `Error`, `Result`, `DEFAULT_IGNORE_PATTERNS`,
`default_ignore_patterns`.

Full API reference: <https://rustywatch.vercel.app/reference/library-api/> and
<https://docs.rs/rustywatch>.

## Help

```
rustywatch --help
```

## Update version

```shell
cargo install rustywatch --force
```

## Testing

### Run all tests

```shell
cargo test
```

### Run unit tests only

```shell
cargo test --lib
```

### Run integration tests only

```shell
cargo test --test '*'
```

### Run doc tests only

> `cargo test --all-targets` skips these, so CI runs them as a separate step.

```shell
cargo test --doc
```

### Run with code coverage

> Requires `cargo-llvm-cov`: `cargo install cargo-llvm-cov`

```shell
cargo llvm-cov
```

### Run benchmarks

```shell
cargo bench
```

## Support languages

Any language — the watcher only runs commands. These are the ones `rustywatch
init` auto-detects and scaffolds:

| Language | Marker file |
|----------|-------------|
| Rust | `Cargo.toml` |
| Go | `go.mod` |
| Bun | `bun.lockb` |
| Node.js | `package.json` |
| Python | `pyproject.toml`, `setup.py`, `requirements.txt` |

## Star History

<a href="https://star-history.com/#ak9024/rustywatch&Timeline">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=ak9024/rustywatch&type=Timeline&theme=dark" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=ak9024/rustywatch&type=Timeline" />
   <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=ak9024/rustywatch&type=Timeline" />
 </picture>
</a>

## License

MIT & Apache-2.0
