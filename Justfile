set shell := ["C:/Program Files/Git/bin/bash.exe", "-c"]

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

# Run all tests across the workspace
test:
    cargo test --workspace

# Run tests for a specific crate: just test-crate rin-core
test-crate crate:
    cargo test -p {{crate}}

# Format the entire codebase (.rs and .toml files)
fmt:
    cargo fmt --all
    taplo format

# Run Clippy linter
lint:
    cargo clippy --workspace -- -D warnings

# Full quality gate: format, lint, check, test
qa:
    @just fmt
    @just lint
    @just check
    @just test

# Kill all stuck cargo and rustc processes
kill:
    -kill -9 $(ps -W | grep -i cargo.exe | awk '{print $1}') 2>/dev/null
    -kill -9 $(ps -W | grep -i rustc.exe | awk '{print $1}') 2>/dev/null

# Clean Cargo cache
clean:
    cargo clean
