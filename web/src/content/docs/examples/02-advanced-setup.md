---
title: Advanced setup
description: Advanced setup page.
---

Examples to run multi project directories in single command `rustywatch`.

```shell
workspaces:
  - dir: "go-project"
    cmd:
    # copy .env to root
    - cp ./go-project/.env .env
    - |
      cd ./go-project;
      go build main.go
    bin_path: "go-project/main"
    bin_arg:
    - server

  - dir: "nodejs-project"
    cmd:
    - |
      cd nodejs-project;
      node index.js
  
  - dir: "rust-project"
    cmd: "cd rust-project;cargo build"
    bin_path: "rust-project/target/debug/rust-project"
    ignore:
    - "target/"
```

```shell
.
├── go-project
│   ├── go.mod
│   ├── go.sum
│   └── main.go
├── nodejs-project
│   ├── index.js
│   └── package.json
├── rust-project
│   ├── Cargo.lock
│   ├── Cargo.toml
│   ├── src
└── rustywatch.yaml

```

In `go-project` need to add cmd to copy .env from the project to root.

```yaml
workspaces:
- dir: ..
  cmd:
    # copy .env to root
    - cp ./go-project/.env .env
    - |
        cd ./go-project;
        go build main.go
```

RustyWatch run on your root directory, then execute your binary from the root, so we need to copy .env from the project to the root.

In `rust-project` need to add `ignore` for `target/` directories.

```yaml
workspaces:
  - dir: ...
    ignore:
    - "target/"
```

Target directories in Rust always changes, so we need to ignore them.
