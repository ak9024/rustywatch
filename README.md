# RustyWatch

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/ak9024/rustywatch/ci.yml) ![Crates.io License (version)](https://img.shields.io/crates/l/rustywatch/0.1.0) ![Crates.io Size](https://img.shields.io/crates/size/rustywatch) ![Crates.io Downloads (version)](https://img.shields.io/crates/dv/rustywatch/0.1.0)

The High-Performance File Monitoring Tool for DevOps Automation

RustyWatch is a robust, Rust-powered file monitoring CLI tool built for developers and DevOps teams who need reliable, high-performance file change detection and automation. Whether you’re running critical services in production, testing, or deploying on local or remote environments, RustyWatch is your lightweight solution to effortlessly track changes and trigger custom workflows.

## Install

```bash
cargo install rustywatch
```

## Usage

```bash
rustywatch -d . -c "echo 'Files changes!'"
```

## License

MIT & Apache-2.0
