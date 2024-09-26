---
title: Reference
description: A reference page
---

## CLI Reference

```shell
rustywatch -h
```

| **Option**   | **Description**                                                | **Type**    |
|--------------|----------------------------------------------------------------|-------------|
| `--dir`      | Specifies the configuration directory to be monitored by RustyWatch. | `string`    |
| `--cmd`      | Defines the command(s) to be executed.                         | `string`    |
| `--bin_path` | Specifies the path to the binary to execute, invoked after `--cmd`. | `string`    |
| `--bin_arg`  | Allows you to provide additional arguments for the binary.     | `array`     |
| `--ignore`   | Lists directories or files to be ignored.                      | `array`     |
| `--cmd`      | Custom command to be executed.                                 | `string`    |

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
| `dir`         | Specifies the configuration directory to be monitored by RustyWatch. | `string`             |
| `cmd`         | Defines the command(s) to be executed. Can be a string or an array. | `string` \| `array`  |
| `bin_path`    | Specifies the path to the binary to execute, invoked after `cmd`.    | `string`             |
| `bin_arg`     | Allows additional arguments for the binary to be provided.        | `array`              |
| `ignore`      | Lists directories or files to be ignored.                        | `array`              |

