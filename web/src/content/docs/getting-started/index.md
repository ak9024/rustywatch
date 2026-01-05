---
title: Getting Started
---

**RustyWatch** Now Supports Multiple Projects! 🤘

:::note

**What's New?**

- Run Multiple Projects Simultaneously: No need for complex setups—RustyWatch now handles multiple projects effortlessly.
- No Dependencies: Forget about monorepos, TurboRepo, or any additional tools. All you need is RustyWatch!
- Automatic Working Directory: Commands execute in the workspace `dir` automatically—no `cd` prefix needed!

:::

**Installation**

> Open your terminal and run:

```shell
cargo install rustywatch
```

**Configuration**

in the root directory of your projects. create file named `rustywatch.yaml`.

```yaml
workspaces:
  # define your directory here.
  - dir: go-project
    # commands run automatically in 'go-project/' directory
    cmd: go build
    # add your binary location
    bin_path: 'go-project/go-project'
    # optional: if do you have a specific arguments
    bin_arg:
      - server # go run main.go server
    # ignore or skip to listen from RustyWatch
    ignore:
      - '.git/'
    # optional: load environment variables from .env file
    env_file: '.env'

  # Add another projects
  - dir: <your_dir>
    cmd: <your_commands>
    bin_path: <your_bin_location>
    bin_arg: <your_bin_arguments>
    ignore: <ignore>
    env_file: <path_to_env_file>
```

:::tip
Commands automatically execute in the workspace directory. No need for `cd` prefixes!
:::

**Usage**

Once you've set up the configuration file, simply run RustyWatch by typing:

```shell
rustywatch
```

RustyWatch will automatically start building your projects based on the settings in your `rustywatch.yaml`
