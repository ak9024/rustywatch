# RustyWatch

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/ak9024/rustywatch/cd.yml?style=flat&label=deployment) 
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/ak9024/rustywatch/ci.yml?branch=main&style=plastic&label=lint) ![Crates.io Total Downloads](https://img.shields.io/crates/d/rustywatch) 
![Crates.io License](https://img.shields.io/crates/l/rustywatch) 
![docs.rs](https://img.shields.io/docsrs/rustywatch?style=social) ![Crates.io Size](https://img.shields.io/crates/size/rustywatch?style=flat) ![GitHub Repo stars](https://img.shields.io/github/stars/ak9024/rustywatch) 
![GitHub Tag](https://img.shields.io/github/v/tag/ak9024/rustywatch) 
![Crates.io Version](https://img.shields.io/crates/v/rustywatch) 
![Codecov](https://img.shields.io/codecov/c/github/ak9024/rustywatch)

[![asciicast](https://asciinema.org/a/677076.svg)](https://asciinema.org/a/677076)

Live reloading inspired by [Go Air](https://github.com/air-verse/air/tree/master) build with Rust.

## Features

- Universal live reloading support for all programming languages
- Real-time binary reloading
- Monorepo development support
- Run multiple projects with a single command
- Optimized and efficient build process
- Automatic detection and monitoring of new directories
- Enhanced, colorful, and detailed log output

## Install

> curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

```shell
cargo install rustywatch
```

## Usage

To start the project with `rustywatch` just need `rustywatch.yaml` for configuration, and run the CLI in root your project.

### Configuration

Default configuration named `rustywatch.yaml`, and please put the config in your root directory, for reference please check at bellow:

```yaml
# define workspaces, rustywatch can be handled multi project at the same time.
workspaces:
  # first project binary apps
  - dir: 'golang-project' # define path directory
    cmd: # define command to build binary
    - cd ./golang-project
    - go build main.go
    bin_path: './golang-projec/main' # define path for binary location
    bin_arg: # define arguments
     - server
    ignore:
     - '.git'
  # second project binary apps
  - dir: 'rust-project'
    cmd:
    - cd ./rust-project
    - cargo build
    bin_path: './rust-project/target/debug/rust-project'
  # third project not binary apps
  - dir: 'nodejs-project'
    cmd: 'cd nodejs-project;npm run dev'
  # more ...
```

```shell
# list directories
ls 
.
└── your-project/
    ├── go-project/
    │   ├── go.mod
    │   ├── go.sum
    │   └── main.go
    ├── rust-project/
    │   ├── src/
    │   │   └── main.rs
    │   ├── Cargo.toml
    │   └── Cargo.lock
    ├── nodejs-project/
    │   ├── index.js
    │   ├── package.json
    │   └── package-lock.json
    └── rustywatch.yaml (config here)
```

### Run the project

```shell
rustywatch
```

## Help

```
rustywatch --help
```

## Support languages

- NodeJS
- Go
- Rust
- Javascript
- (more)

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=ak9024/rustywatch&type=Date)](https://star-history.com/#ak9024/rustywatch&Date)
## License

MIT & Apache-2.0
