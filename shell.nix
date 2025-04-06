{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    # Rust toolchain
    rustc
    cargo
    rustfmt
    clippy
    rust-analyzer
    
    # Build essentials
    pkg-config
    openssl
    openssl.dev
    
    # Development tools
    lldb
    gdb
    
    # Version control
    git
    
    # Documentation
    rustup
  ];

  shellHook = ''
    echo "Rust development environment setup!"
    export RUST_BACKTRACE=1
  '';

  # Set environment variables if needed
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
