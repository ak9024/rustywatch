---
title: Reference
description: A reference page
---

## CLI Reference

```shell
rustywatch -h
```

### Watch Options

| **Option**   | **Description**                                                | **Type**    |
|--------------|----------------------------------------------------------------|-------------|
| `--dir`      | Specifies the configuration directory to be monitored by RustyWatch. | `string`    |
| `--cmd`      | Defines the command(s) to be executed.                         | `string`    |
| `--bin_path` | Specifies the path to the binary to execute, invoked after `--cmd`. | `string`    |
| `--bin_arg`  | Allows you to provide additional arguments for the binary.     | `array`     |
| `--ignore`   | Lists directories or files to be ignored.                      | `array`     |
| `--monitor`  | Launch with the TUI dashboard for real-time process monitoring. | `flag`     |

### Init Command

```shell
rustywatch init [OPTIONS]
```

| **Option**   | **Description**                                                | **Type**    |
|--------------|----------------------------------------------------------------|-------------|
| `-o, --output` | Config file path (default: `rustywatch.yaml`)                | `string`    |
| `--yes`      | Skip prompts and use auto-detected defaults                    | `flag`      |
| `-f, --force`| Overwrite existing config file                                 | `flag`      |

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
| `cmd`         | Defines the command(s) to be executed. Can be a string or an array. No `cd` prefix needed. | `string` \| `array`  |
| `bin_path`    | Path to the binary to execute, **relative to the workspace directory**. E.g., `./target/debug/app` for Rust. | `string`             |
| `bin_arg`     | Allows additional arguments for the binary to be provided.        | `array`              |
| `ignore`      | Lists directories or files to be ignored. Paths relative to workspace dir. | `array`              |
| `env_file`    | Path to `.env` file for loading environment variables. Relative to workspace dir or absolute from root (prefix with `/`). | `string`             |

