---
title: Quickstart
description: Quickstart page.
---

## Install

Please install cargo first `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh` if you don't have.

```shell
cargo install rustywatch
```

The default configuration file is named `rustywatch.yaml`, and it must be located in your project's root directory. For a reference configuration, please see the example below:

> Please suitable with your project configuration.

```yaml
# define workspaces, rustywatch can be handled multi project at the same time.
workspaces:
  # first project binary apps
  - dir: 'golang-project' # define path directory
    cmd: # define command to build binary
    - cp ./golang-project/.env .env
    - |
      cd ./golang-project;
      go build main.go
    bin_path: './golang-projec/main' # define path for binary location
    bin_arg: # define arguments
     - server
    ignore:
     - '.git'
  # second project binary apps
  - dir: 'rust-project'
    cmd:
    - |
      cd ./rust-project;
      cargo build
    bin_path: './rust-project/target/debug/rust-project'
  # third project non binary apps
  - dir: 'nodejs-project'
    cmd: 'cd nodejs-project;npm run dev'
  # more ...
```

If done with the configuration open your shell then type `rustywatch`.

```shell
╭━━━╮╱╱╱╱╱╭╮╱╱╱╱╭╮╭╮╭╮╱╱╭╮╱╱╱╭╮
┃╭━╮┃╱╱╱╱╭╯╰╮╱╱╱┃┃┃┃┃┃╱╭╯╰╮╱╱┃┃
┃╰━╯┣╮╭┳━┻╮╭╋╮╱╭┫┃┃┃┃┣━┻╮╭╋━━┫╰━╮
┃╭╮╭┫┃┃┃━━┫┃┃┃╱┃┃╰╯╰╯┃╭╮┃┃┃╭━┫╭╮┃
┃┃┃╰┫╰╯┣━━┃╰┫╰━╯┣╮╭╮╭┫╭╮┃╰┫╰━┫┃┃┃
╰╯╰━┻━━┻━━┻━┻━╮╭╯╰╯╰╯╰╯╰┻━┻━━┻╯╰╯
╱╱╱╱╱╱╱╱╱╱╱╱╭━╯┃
╱╱╱╱╱╱╱╱╱╱╱╱╰━━╯
[WARN ] Remove old binary: go-project/main
[WARN ] Remove old binary: rust-project/target/debug/rust-project
[WARN ] Old binary removed
[WARN ] Old binary removed
Hello Adiatma Kamarudin
[INFO ] Waching directory: "nodejs-project"
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.01s
[INFO ] Restarting binary: rust-project/target/debug/rust-project
[INFO ] Binary started: 71127
[INFO ] Waching directory: "rust-project"
Hello, world!
[INFO ] Restarting binary: go-project/main
[INFO ] Binary started: 71133
[INFO ] Waching directory: "go-project"

 ┌───────────────────────────────────────────────────┐
 │                   Fiber v2.52.5                   │
 │               http://127.0.0.1:3001               │
 │       (bound on host 0.0.0.0 and port 3001)       │
 │                                                   │
 │ Handlers ............. 4  Processes ........... 1 │
 │ Prefork ....... Disabled  PID ............. 71133 │
 └───────────────────────────────────────────────────┘
```

If do you prefer to run `rustywatch` without configuration.

```shell
rustywatch --dir . --cmd 'go build main.go' --bin-path './main'
```

