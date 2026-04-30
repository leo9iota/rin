default:
    @just --list

# Run the Rin CLI (TUI)
run:
    cargo run -p rin-cli

# Run the Rin API server
run-api:
    cargo run -p rin-api

# Build all workspace members in release mode
build:
    cargo build --workspace --release

# Fast check for compilation errors across the workspace
check:
    cargo check --workspace

# Format the entire workspace (Rust and TOML)
fmt:
    cargo fmt --all
    taplo format

# Run Clippy linter across the workspace
lint:
    cargo clippy --workspace -- -D warnings

# Clean Cargo cache
clean:
    cargo clean
