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
    bin_path: './go-project'  # relative to workspace dir
    env_file: '.env'
  - dir: 'examples/nodejs-project'
    cmd: npm start
    env_file: '/.env'  # loads from project root
  - dir: 'examples/rust-project'
    cmd:
      - echo "Building rust-project..."
      - cargo build
    bin_path: './target/debug/rust-project'  # relative to workspace dir
    ignore:
      - 'target/'
```

:::tip
Commands automatically execute in the workspace `dir` directory. No need for `cd` prefixes!
:::

**Explanation the configuration.**

- `dir`: The project directory to watch. **Commands automatically execute in this directory.**
- `cmd`: The commands to run for each project. Can be a single command or an array of commands.
- `bin_path`: The path to the binary executable file, **relative to the workspace directory**. For example, in a Rust project with `dir: 'my-app'`, use `bin_path: './target/debug/my-app'` instead of `bin_path: 'my-app/target/debug/my-app'`.
- `ignore` (optional): Files or directories that should be excluded from being monitored by RustyWatch. Paths are relative to the workspace directory.
- `env_file` (optional): Path to a `.env` file containing environment variables. Use a relative path (e.g., `.env`) to load from the workspace directory, or prefix with `/` (e.g., `/.env`) to load from the project root.


**Example Projects**

**Go Project:**

- Located in `examples/go-project`
- Commands: `go build` (runs in `examples/go-project/`)
- Binary location: `./go-project` (relative to workspace dir)

**Node.js Project:**

- Located in `examples/nodejs-project`
- Commands: `npm start` (runs in `examples/nodejs-project/`)

**Rust Project:**

- Located in `examples/rust-project`
- Commands: `cargo build` (runs in `examples/rust-project/`)
- Binary location: `./target/debug/rust-project` (relative to workspace dir)
- The `target/` directory is ignored to prevent unnecessary rebuilds.

**Usage**

Once you have your `rustywatch.yaml` configured. simply run:

```shell
rustywatch
```

RustyWatch will handle everything automatically, building and watching your multiple projects as defined in the configuration file.

For more examples please check this out: https://github.com/ak9024/rustywatch/tree/main/examples.

