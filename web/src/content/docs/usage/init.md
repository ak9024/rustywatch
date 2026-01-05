---
title: Init Command
sidebar:
  order: 0
---

## Quick Start with Init

The `rustywatch init` command creates a configuration file interactively, making it easy to get started with RustyWatch.

### Basic Usage

```shell
rustywatch init
```

This launches an interactive wizard that:
1. Detects your project type automatically
2. Prompts for watch directory, build commands, and binary path
3. Asks about ignore patterns
4. Shows a preview of the configuration
5. Writes the `rustywatch.yaml` file

### Command Options

| Flag | Description |
|------|-------------|
| `-o, --output <FILE>` | Config file path (default: `rustywatch.yaml`) |
| `--yes` | Skip prompts and use auto-detected defaults |
| `-f, --force` | Overwrite existing config file |

### Auto-Detection

RustyWatch automatically detects your project type and suggests appropriate defaults:

| Project Type | Detection File | Default Command | Default Binary |
|-------------|----------------|-----------------|----------------|
| **Rust** | `Cargo.toml` | `cargo build` | `target/debug/{name}` |
| **Go** | `go.mod` | `go build` | `./{name}` |
| **Node.js** | `package.json` | `npm run dev` | - |
| **Python** | `pyproject.toml`, `setup.py`, `requirements.txt` | `python main.py` | - |
| **Bun** | `bun.lockb` | `bun run start` | - |

### Examples

**Interactive mode (recommended for first-time setup):**

```shell
rustywatch init
```

**Quick setup with auto-detected defaults:**

```shell
rustywatch init --yes
```

**Custom output path:**

```shell
rustywatch init -o .rustywatch.yaml
```

**Force overwrite existing config:**

```shell
rustywatch init --force
```

### Interactive Prompts

When running `rustywatch init`, you'll be asked:

1. **Watch Directory** - Which directory should RustyWatch monitor for changes?
2. **Build Command** - What command should run when files change?
3. **Binary Path** - Where is the compiled binary located? (for compiled languages)
4. **Ignore Patterns** - Which files/directories should be ignored?

After answering, you'll see a preview of the generated YAML configuration before it's written to disk.

:::tip
Use `--yes` flag for CI/CD pipelines or when you trust the auto-detected defaults.
:::
