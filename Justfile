default:
    @just --list

# Run the Rin CLI
run:
    cargo run -p rin-cli

# Run the Rin API server
run-api:
    cargo run -p rin-api

# Build all codebase members in release mode
build:
    cargo build --workspace --release

# Fast check for compilation errors
check:
    cargo check --workspace

# Format the entire codebase (.rs and .toml files)
fmt:
    cargo fmt --all
    taplo format

# Run Clippy linter
lint:
    cargo clippy --workspace -- -D warnings

# Clean Cargo cache
clean:
    cargo clean
