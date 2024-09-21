# RustyWatch

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/ak9024/rustywatch/cd.yml?style=flat&label=deployment) 
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/ak9024/rustywatch/ci.yml?branch=main&style=plastic&label=lint) ![Crates.io Total Downloads](https://img.shields.io/crates/d/rustywatch) 
![Crates.io License](https://img.shields.io/crates/l/rustywatch) 
![docs.rs](https://img.shields.io/docsrs/rustywatch?style=social) ![Crates.io Size](https://img.shields.io/crates/size/rustywatch?style=flat) ![GitHub Repo stars](https://img.shields.io/github/stars/ak9024/rustywatch) 
![GitHub Tag](https://img.shields.io/github/v/tag/ak9024/rustywatch) 
![Crates.io Version](https://img.shields.io/crates/v/rustywatch) 
![Codecov](https://img.shields.io/codecov/c/github/ak9024/rustywatch)

```shell

╭━━━╮╱╱╱╱╱╭╮╱╱╱╱╭╮╭╮╭╮╱╱╭╮╱╱╱╭╮
┃╭━╮┃╱╱╱╱╭╯╰╮╱╱╱┃┃┃┃┃┃╱╭╯╰╮╱╱┃┃
┃╰━╯┣╮╭┳━┻╮╭╋╮╱╭┫┃┃┃┃┣━┻╮╭╋━━┫╰━╮
┃╭╮╭┫┃┃┃━━┫┃┃┃╱┃┃╰╯╰╯┃╭╮┃┃┃╭━┫╭╮┃
┃┃┃╰┫╰╯┣━━┃╰┫╰━╯┣╮╭╮╭┫╭╮┃╰┫╰━┫┃┃┃
╰╯╰━┻━━┻━━┻━┻━╮╭╯╰╯╰╯╰╯╰┻━┻━━┻╯╰╯
╱╱╱╱╱╱╱╱╱╱╱╱╭━╯┃
╱╱╱╱╱╱╱╱╱╱╱╱╰━━╯
version: v0.1.5


Live reloading for any programing languages

Usage: rustywatch [OPTIONS] --cmd <COMMAND>

Options:
  -d, --dir <DIR>            [default: .]
  -c, --cmd <COMMAND>
  -i, --ignore <IGNORE>
      --bin-path <BIN_PATH>
  -h, --help                 Print help
  -V, --version              Print version
```

Live reloading inspired by [Go Air](https://github.com/air-verse/air/tree/master) build with Rust.

## Features

- Live reloading support for all programming languages
- Real-time reloading for binaries
- Optimized build process
- Automatic detection and monitoring of new directories
- Enhanced, colorful log output

## Install

> curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

```shell
cargo install rustywatch
```

## Usage

```shell
rustywatch -d . -c "echo 'Files changes!'" --ignore .git --ignore ./tmp
```

- Example using with Rust (cargo)

```shell
# create new project
cargo new hello-world;
cd hello-world;
# run rustywatch in `./src` directory
rustywatch -d './src' -c 'cargo run'
```

- Example using with Go
```shell
# create new project
mkdir hello-world;
cd hello-world;
# init go module
go mod init go-project;
# create file main.go
touch main.go;
# edit file main.go
vim main.go
# and run the project with rustywatch
# then the project will be running with hot reload.
rustywatch -d . -c 'go run main.go' --ignore .git
```

- Example using with Go (Fiber)

```shell
mkdir go-fiber;
cd go-fiber;
go mod init go-fiber;
# install fiber framework
go get github.com/gofiber/fiber/v2
# start live reload with rustywatch
rustywatch -d . -c 'go build main.go' --bin-path './main' --bin-arg server
```

- Example using with NodeJS
```shell
# create new project
mkdir hello-world;
cd nodejs-project;
# create index.js
touch index.js
# edit file index.js and and run the project with rustywatch 
vim index.js
# then the project will be running with hot reload
rustywatch -d . -c 'node index.js' --ignore '.tmp' --ignore '.git'
```

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=ak9024/rustywatch&type=Date)](https://star-history.com/#ak9024/rustywatch&Date)
## License

MIT & Apache-2.0
