---
title: Reference
description: A reference page
---

Using RustyWatch as a Rust crate instead of a CLI? See the
[Library API](/reference/library-api/) reference.

## CLI Reference

```shell
rustywatch -h
```

### Watch Options

| **Option**   | **Description**                                                | **Type**    |
|--------------|----------------------------------------------------------------|-------------|
| `-d, --dir`      | Directory to watch; also the working directory for `--cmd`. Defaults to `.`. | `string`    |
| `-c, --cmd`      | Defines the command(s) to be executed. Repeatable; multiple commands run in parallel. | `string`    |
| `--bin-path` | Specifies the path to the binary to execute, invoked after `--cmd`. | `string`    |
| `--bin-arg`  | Allows you to provide additional arguments for the binary.     | `array`     |
| `-i, --ignore`   | Lists directories or files to be ignored.                      | `array`     |
| `--cfg`      | Config file path. Defaults to `rustywatch.yaml`; when it exists it takes precedence over the flags above. | `string`    |
| `--monitor`  | Launch with the TUI dashboard for real-time process monitoring. | `flag`     |

Without a config file, `--cmd` is required — RustyWatch errors instead of
watching with nothing to run.

### Init Command

```shell
rustywatch init [OPTIONS]
```

| **Option**   | **Description**                                                | **Type**    |
|--------------|----------------------------------------------------------------|-------------|
| `-o, --output` | Config file path (default: `rustywatch.yaml`)                | `string`    |
| `-d, --dir`  | Root directory to scan for projects (default: `.`)             | `string`    |
| `--depth`    | How deep to scan for nested projects (default: `2`, `0` = root only) | `integer` |
| `--yes`      | Skip prompts and accept every detected project as-is           | `flag`      |
| `-f, --force`| Overwrite existing config file                                 | `flag`      |
| `--dry-run`  | Print the configuration instead of writing it                  | `flag`      |

`init` scans subdirectories, so a monorepo generates all its workspaces in one
run. See [Init Command](/usage/init/) for details.

##### Workspaces

All commands define under `workspaces` field.

Example.

```yaml
workspaces:
 - <field_options>
 # more
```

All field under `workspaces` are using `array`:


#### Field Options:

| **Field**     | **Description**                                                  | **Type**             |
|---------------|------------------------------------------------------------------|----------------------|
| `dir`         | The workspace directory to watch. **Commands execute automatically in this directory.** | `string`             |
| `cmd`         | Defines the command(s) to be executed. Can be a string or an array. No `cd` prefix needed. Array entries run **in parallel**, not in sequence — chain with `&&` in one string when order matters. | `string` \| `array`  |
| `bin_path`    | Path to the binary to execute, **relative to the workspace directory**. E.g., `./target/debug/app` for Rust. | `string`             |
| `bin_arg`     | Allows additional arguments for the binary to be provided.        | `array`              |
| `ignore`      | Lists directories or files to be ignored. Paths relative to workspace dir. Supports globs (`*.log`), directories (`target/`) and plain names. A non-empty list **replaces** the built-in defaults (`target/`, `node_modules/`, `.git/`, …) rather than extending them. | `array`              |
| `env_file`    | Path to `.env` file for loading environment variables. Relative to workspace dir or absolute from root (prefix with `/`). | `string`             |

