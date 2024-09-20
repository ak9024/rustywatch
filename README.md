# RustyWatch

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/ak9024/rustywatch/ci.yml) ![Crates.io License (version)](https://img.shields.io/crates/l/rustywatch/0.1.0) ![Crates.io Size](https://img.shields.io/crates/size/rustywatch) ![Crates.io Downloads (version)](https://img.shields.io/crates/dv/rustywatch/0.1.0)

[![asciicast](https://asciinema.org/a/PAKBZOON6TZgbgS41dU73Niq5.svg)](https://asciinema.org/a/PAKBZOON6TZgbgS41dU73Niq5)

Live reloading inspired by https://github.com/air-verse/air/tree/master build with Rust.

## Features

- Live reloading for any programing languages
- Better building process
- Lite just 16.9 kB binary
- Allow watching new directories
- Colorful log output

## Install

```shell
cargo install rustywatch
```

## Usage

```shell
━━━╮╱╱╱╱╱╭╮╱╱╱╱╭╮╭╮╭╮╱╱╭╮╱╱╱╭╮
┃╭━╮┃╱╱╱╱╭╯╰╮╱╱╱┃┃┃┃┃┃╱╭╯╰╮╱╱┃┃
┃╰━╯┣╮╭┳━┻╮╭╋╮╱╭┫┃┃┃┃┣━┻╮╭╋━━┫╰━╮
┃╭╮╭┫┃┃┃━━┫┃┃┃╱┃┃╰╯╰╯┃╭╮┃┃┃╭━┫╭╮┃
┃┃┃╰┫╰╯┣━━┃╰┫╰━╯┣╮╭╮╭┫╭╮┃╰┫╰━┫┃┃┃
╰╯╰━┻━━┻━━┻━┻━╮╭╯╰╯╰╯╰╯╰┻━┻━━┻╯╰╯
╱╱╱╱╱╱╱╱╱╱╱╱╭━╯┃
╱╱╱╱╱╱╱╱╱╱╱╱╰━━╯


version: v0.1.3
Live reload for any programing languages

Usage: rustywatch [OPTIONS] --cmd <COMMAND>

Options:
  -d, --dir <DIR>        [default: .]
  -c, --cmd <COMMAND>
  -i, --ignore <IGNORE>
  -h, --help             Print help
  -V, --version          Print version
```

```shell
rustywatch -d . -c "echo 'Files changes!'" --ignore .git --ignore ./tmp
```

- Example using with Go
```shell
mkdir go-project;
cd go-project;
go mod init go-project;
touch main.go;
# edit file go.
vim main.go
# and run the project with rustywatch
# then the project will be running with hot reload.
rustywatch -d . -c 'go run main.go'
```

- Example using NodeJS
```shell
mkdir nodejs-project
cd nodejs-project;
touch index.js
# edit file index.js and and run the project with rustywatch 
vim index.js
# then the project will be running with hot reload
rustywatch -d . -c 'node index.js'
```

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=ak9024/rustywatch&type=Date)](https://star-history.com/#ak9024/rustywatch&Date)
## License

MIT & Apache-2.0
