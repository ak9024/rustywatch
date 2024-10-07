---
title: Getting Started
---

**RustyWatch** Now Supports Multiple Projects! 🤘 

**What's New?**

- Run Multiple Projects Simultaneously: No need for complex setups—RustyWatch now handles multiple projects effortlessly.
- No Dependencies: Forget about monorepos, TurboRepo, or any additional tools. All you need is RustyWatch!

```shell
cargo install rustywatch
```

Just create `rustywatch.yaml` in your root of projects, then type `rustywatch` in your shell for running.

```yaml
workspaces:
  # define your directory here.
  - dir: go-project
    # add command to build your project.
    cmd:
      - go build
    # add your binary location
    bin_path: 'go-project/go-project'
    # optional: if do you have a specific arguments
    bin_arg:
      - server # go run main.go server
    # ignore or skip to listen from RustyWatch
    ignore:
      - '.git/'

  # Add another projects
  - dir: <your_dir>
    cmd: <your_commands>
    bin_path: <your_bin_location>
    bin_arg: <your_bin_arguments>
    ignore: <ignore>
```
