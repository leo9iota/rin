default:
    @just --list

# Run the Rin Indexer natively
run:
    cargo run

# Run the Rin Indexer with full optimizations
run-release:
    cargo run --release

# Fast check for compilation errors
check:
    cargo check

# Format the codebase
fmt:
    cargo fmt

# Run the linter (Clippy)
lint:
    cargo clippy -- -D warnings

# Clean the cargo cache
clean:
    cargo clean
