default:
    @just --list

# Run Rin Indexer natively
run:
    cargo run

# Run Rin Indexer with full optimizations
run-release:
    cargo run --release

# Fast check for compilation errors
check:
    cargo check

# Format codebase
fmt:
    cargo fmt

# Run Clippy linter
lint:
    cargo clippy -- -D warnings

# Clean Cargo cache
clean:
    cargo clean
