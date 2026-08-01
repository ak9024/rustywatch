---
title: Init Command
sidebar:
  order: 0
---

## Quick Start with Init

The `rustywatch init` command scans your project tree and writes a
configuration file, so a monorepo is set up in one step.

### Basic Usage

```shell
rustywatch init
```

This:

1. Scans the current directory and its subdirectories for known project markers
2. Lists everything found and lets you pick which projects to watch
3. Fills in the command, binary path, ignore patterns and `.env` file for each
4. Optionally walks you through customizing every field
5. Validates the result and shows a preview before writing `rustywatch.yaml`

Nothing is written until the config passes the same validation `rustywatch`
itself applies, so `init` cannot leave you with a file that fails to load.

### Command Options

| Flag | Description |
|------|-------------|
| `-o, --output <FILE>` | Config file path (default: `rustywatch.yaml`) |
| `-d, --dir <DIR>` | Root directory to scan (default: `.`) |
| `--depth <N>` | How deep to scan for nested projects (default: `2`; `0` = only the root) |
| `--yes` | Skip prompts and accept every detected project as-is |
| `-f, --force` | Overwrite an existing config file |
| `--dry-run` | Print the configuration instead of writing it |

`--yes` never blocks on a prompt: if the output file already exists it errors
and asks for `--force` rather than waiting for an answer. Running `init`
without `--yes` outside a terminal errors for the same reason.

### Multi-project scanning

Given this tree:

```
shop/
├── Cargo.toml            # Rust
├── services/
│   └── api/go.mod        # Go
├── web/
│   ├── package.json      # Node.js
│   └── .env
└── node_modules/         # skipped
```

`rustywatch init --yes` produces all three workspaces at once:

```yaml
workspaces:
- dir: .
  cmd: cargo build
  ignore:
  - target/
  - .git/
  bin_path: ./target/debug/shop
- dir: services/api
  cmd: go build
  ignore:
  - vendor/
  - .git/
  bin_path: ./api
- dir: web
  cmd: npm run dev
  ignore:
  - node_modules/
  - .git/
  - dist/
  - .next/
  env_file: .env
```

Build output and dependency directories (`target/`, `node_modules/`, `vendor/`,
`dist/`, `build/`, `out/`, `coverage/`, `__pycache__/`, `venv/`) and hidden
directories are never descended into, so vendored copies of a project are not
mistaken for the real thing.

Use `--depth 0` to look only at the root, or raise it for deeper trees.

### Auto-Detection

RustyWatch detects the project type and reads the manifests to pick defaults
that point at things which actually exist:

| Project Type | Detection File | Default Command | Default Binary |
|-------------|----------------|-----------------|----------------|
| **Rust** | `Cargo.toml` | `cargo build` | `./target/debug/{name}` |
| **Go** | `go.mod` | `go build` | `./{name}` |
| **Node.js** | `package.json` | first of `dev`/`start`/`serve`/`watch` in `scripts`, else `npm run dev` | - |
| **Bun** | `bun.lockb` | same, run with `bun` | - |
| **Python** | `pyproject.toml`, `setup.py`, `requirements.txt` | `python <main/app/manage/run/__main__>.py` | - |
| **Other** | - | placeholder to edit | - |

The project name comes from `Cargo.toml`, `package.json` or `go.mod`, falling
back to the directory name. When a workspace has a `.env` file it is wired up as
`env_file` automatically.

:::note
The watch directory for a Rust project is the crate root, not `src/`. A relative
`bin_path` resolves against the watch directory, so watching `src/` would send
RustyWatch looking for `src/target/debug/{name}`.
:::

### Examples

**Interactive setup:**

```shell
rustywatch init
```

**Accept everything detected:**

```shell
rustywatch init --yes
```

**Preview without writing:**

```shell
rustywatch init --yes --dry-run
```

**Scan a subdirectory only:**

```shell
rustywatch init --dir services --depth 1
```

**Custom output path:**

```shell
rustywatch init -o .rustywatch.yaml
```

**Force overwrite existing config:**

```shell
rustywatch init --yes --force
```

### Interactive Prompts

When projects are detected you are asked:

1. **Which projects to watch** — a multi-select, everything pre-selected
   (space toggles, enter confirms)
2. **Whether to customize** — answer no to keep the detected settings

Choosing to customize (or having nothing detected) walks each workspace through:

1. **Watch directory** — must be an existing directory
2. **Build/run command** — required, runs inside the watch directory
3. **Binary path** — leave empty for interpreted projects
4. **Binary arguments** — space-separated, only asked when a binary is set
5. **Ignore patterns** — comma-separated, rejected if a pattern is not a valid glob
6. **Env file** — leave empty for none

You can then add further workspaces by hand before the preview is shown.

:::tip
Use `--yes` for CI/CD pipelines or when you trust the auto-detected defaults,
and pair it with `--dry-run` to see the result without touching disk.
:::
