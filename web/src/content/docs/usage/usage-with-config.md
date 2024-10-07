---
title: Usage with config
---

```shell
.
├── rustywatch.yaml
└── examples/
    ├── go-project
    ├── rust-project
    └── nodejs-project
```

```yaml
workspaces:
  - dir: 'examples/go-project'
    cmd:
      - echo "go-project"
      - |
        cd examples/go-project;
        go build;
    bin_path: 'examples/go-project/go-project'
  - dir: 'examples/nodejs-project'
    cmd:
      - echo "nodejs-project"
      - cd examples/nodejs-project;npm start;
  - dir: 'examples/rust-project'
    cmd:
      - echo "rust-project"
      - |
        cd examples/rust-project;
        cargo build;
    bin_path: 'examples/rust-project/target/debug/rust-project'
    ignore:
      - 'examples/rust-project/target/'
```
