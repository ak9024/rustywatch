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
      - echo "Building go-project..."
      - go build
    bin_path: 'examples/go-project/go-project'
    env_file: '.env'
  - dir: 'examples/nodejs-project'
    cmd: npm start
    env_file: '/.env'
  - dir: 'examples/rust-project'
    cmd:
      - echo "Building rust-project..."
      - cargo build
    bin_path: 'examples/rust-project/target/debug/rust-project'
    ignore:
      - 'examples/rust-project/target/'
```

:::tip
Commands automatically execute in the workspace `dir` directory. No need for `cd` prefixes!
:::

**Explanation the configuration.**

- `dir`: The project directory to watch. **Commands automatically execute in this directory.**
- `cmd`: The commands to run for each project. Can be a single command or an array of commands.
- `bin_path`: The path to the binary executable file produced after the build.
- `ignore` (optional): Files or directories that should be excluded from being monitored by RustyWatch. For example, the Rust project's target directory is ignored to avoid unnecessary rebuilds.
- `env_file` (optional): Path to a `.env` file containing environment variables. Use a relative path (e.g., `.env`) to load from the workspace directory, or prefix with `/` (e.g., `/.env`) to load from the project root.


**Example Projects**

**Go Project:**

- Located in `examples/go-project`
- Commands: `go build` (runs in `examples/go-project/`)
- Binary location: `examples/go-project/go-project`

**Node.js Project:**

- Located in `examples/nodejs-project`
- Commands: `npm start` (runs in `examples/nodejs-project/`)

**Rust Project:**

- Located in `examples/rust-project`
- Commands: `cargo build` (runs in `examples/rust-project/`)
- Binary location: `examples/rust-project/target/debug/rust-project`
- The `target/` directory is ignored to prevent unnecessary rebuilds.

**Usage**

Once you have your `rustywatch.yaml` configured. simply run:

```shell
rustywatch
```

RustyWatch will handle everything automatically, building and watching your multiple projects as defined in the configuration file.

For more examples please check this out: https://github.com/ak9024/rustywatch/tree/main/examples.

