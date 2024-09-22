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

- Live reloading support for all programming languages
- Real-time reloading for binaries
- Support monorepo development
- Run many projects in one commands
- Optimized build process
- Automatic detection and monitoring of new directories
- Enhanced, colorful log output

## Install

> curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

```shell
cargo install rustywatch
```

## Usage

To start the project just type `rustywatch`, but first you need to create the config.

- Config

Create file `rustywatch.yaml` in your root directory.

```yaml
# define workspaces, rustywatch can be handled mutli project at the same time.
workspaces:
  # first project  
  - dir: './golang-project' # define path directory
    cmd: 'cd ./golang-project;go build main.go' # define command to build binary
    bin_path: './golang-projec/main' # define path for binary location
    bin_arg: # define arguments
     - server
    ignore:
     - '.git'
  # second project
  - dir: './rust-project/src/'
    cmd: 'cd ./rust-project/src/;cargo build'
    bin_path: './rust-project/target/debug/rust-project'
  # more ...
```

Run the project

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
- (more) Need to testing

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=ak9024/rustywatch&type=Date)](https://star-history.com/#ak9024/rustywatch&Date)
## License

MIT & Apache-2.0
