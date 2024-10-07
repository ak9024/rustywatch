---
title: Usage with config
---

### Using RustyWatch with configuration.

With RustyWatch, you can manage multiple projects effortlessly using a simple configuration file.

Here's how to setup and use it with a `rustywatch.yaml` file.

**Project structure example**

Your project directory might look like this.

```shell
.
├── rustywatch.yaml
└── examples/
    ├── go-project
    ├── rust-project
    └── nodejs-project
```

**Sample `rustywatch.yaml` configuration.**

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

**Explanation the configuration.**

- `dir`: The project directory to watch.
- `cmd`: The commands to run for each project. You can chain multiple commands using the `|` syntax to write multi-line commands.
- `bin_path`: The path to the binary executable file produced after the build.
- `ignore` (optional): Files or directories that should be excluded from being monitored by RustyWatch. For example, the Rust project's target directory is ignored to avoid unnecessary rebuilds.


**Example Projects**

**Go Project:**

- Located in examples/go-project
- Commands: Builds the Go project using go build
- Binary location: examples/go-project/go-project

**Node.js Project:**

- Located in examples/nodejs-project
- Commands: Runs the Node.js project using npm start

**Rust Project:**

- Located in examples/rust-project
- Commands: Builds the Rust project using cargo build
- Binary location: examples/rust-project/target/debug/rust-project
- The target/ directory is ignored to prevent RustyWatch from listening to changes there.

**Usage**

Once you have your `rustywatch.yaml` configured. simply run:

```shell
rustywatch
```

RustyWatch will handle everything automatically, building and watching your multiple projects as defined in the configuration file.

For more examples please check this out: https://github.com/ak9024/rustywatch/tree/main/examples.

