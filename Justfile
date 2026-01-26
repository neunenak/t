_help:
    @just -l

# Lint the project
lint:
    cargo fmt -- --check
    cargo clippy -- -D warnings
    actionlint

# Format the project
fmt:
    cargo fmt

# Build the project
build:
    cargo build --release

# Test the project
test:
    cargo test --all-features
