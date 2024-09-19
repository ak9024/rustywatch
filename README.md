# rustywatch

A file watcher CLI built with Rust.

## Features

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
