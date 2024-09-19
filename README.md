# RustyWatch

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/ak9024/rustywatch/rust.yml)

The High-Performance File Monitoring Tool for DevOps Automation

RustyWatch is a robust, Rust-powered file monitoring CLI tool built for developers and DevOps teams who need reliable, high-performance file change detection and automation. Whether you’re running critical services in production, testing, or deploying on local or remote environments, RustyWatch is your lightweight solution to effortlessly track changes and trigger custom workflows.

## Install

WIP

## Usage

```bash
# run in development mode.
cargo run -- --dir ./watched_dir --cmd "echo 'Files changes!'"

# release
cargo build --release
cp ./target/debug/rustywatch .
chmod +x rustywatch

rustywatch --dir ./watched_dir --cmd "echo 'Files changes!'"
```

## License

MIT
