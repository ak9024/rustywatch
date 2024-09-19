# RustyWatch

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/ak9024/rustywatch/rust.yml)

The High-Performance File Monitoring Tool for DevOps Automation

RustyWatch is a robust, Rust-powered file monitoring CLI tool built for developers and DevOps teams who need reliable, high-performance file change detection and automation. Whether you’re running critical services in production, testing, or deploying on local or remote environments, RustyWatch is your lightweight solution to effortlessly track changes and trigger custom workflows.

## Why Choose RustyWatch?

Blazing Fast with Rust: Built on the lightning-fast and memory-efficient Rust programming language, RustyWatch ensures that your file monitoring tasks are handled with minimal system overhead. It’s designed for speed, reliability, and low resource consumption—ideal for both local development and production pipelines.

Seamless Automation: Watch any directory or file, and automate actions like restarting services, building code, or syncing files with a simple command-line interface. Whether you're managing CI/CD workflows, auto-compiling projects, or syncing configurations, RustyWatch makes the process seamless.

Cross-Platform Compatibility: Works across all major operating systems (Linux, macOS, Windows). No matter where your code or systems run, RustyWatch provides a consistent, reliable solution for monitoring file changes.

Custom Command Execution: Set up RustyWatch to trigger custom commands when files or directories change. From restarting your server to running deployment scripts, it’s as flexible as your development workflow demands.

Simple, Yet Powerful: With a minimalist design and intuitive interface, RustyWatch provides powerful functionality without unnecessary complexity. The clean, no-frills experience makes it perfect for both seasoned professionals and developers just starting with automation.

Open Source & Community Driven: Backed by the growing Rust ecosystem, RustyWatch is an open-source project with active support. Stay on top of new features and updates with a dedicated community behind the project.

With its superior performance, lightweight design, and cross-platform capabilities, RustyWatch is an indispensable tool for developers, sysadmins, and DevOps professionals who need fast, reliable file monitoring and automation.

Make your workflows smarter, faster, and more efficient with RustyWatch.

## Instalation

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
