---
title: Reference
description: A reference page
---

## CLI Reference

```shell
rustywatch -h
```

| Options    | Description                                                               | Type     |
|------------|---------------------------------------------------------------------------|----------|
| --dir      | Config directory will be listen in RustyWatch.                            | `string` |
| --cmd      | All commands can be defined in here.                                      | `string` |
| --bin_path | Path directory to execute, this field will be call after `cmd`            | `string` |
| --bin_arg  | If your binary need more arguments, can be set in here.                   | `array`  |
| --ignore   | If do you have are `dir` and `files` need to ignore, can be set in here.  | `array`  |
| --cmd      | Custom command                                                            | `string` |## Configuration Reference

Default configuration named `rustywatch.yaml`.

##### Workspaces

All commands define under `workspaces` field.

Example.

```yaml
workspaces:
 - <field_options>
 # more
```

All field under `workspaces` are using `array`:

**Field Options**

| Field    | Description                                                               | Type            |
|----------|---------------------------------------------------------------------------|-----------------|
| dir      | Config directory will be listen in RustyWatch.                            | `string`        |
| cmd      | All commands can be defined in here.                                      | `string\|array` |
| bin_path | Path directory to execute, this field will be call after `cmd`            | `string`        |
| bin_arg  | If your binary need more arguments, can be set in here.                   | `array`         |
| ignore   | If do you have are `dir` and `files` need to ignore, can be set in here.  | `array`         |
