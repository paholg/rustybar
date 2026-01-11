run:
    direnv exec / nix run .

# Generate code and run lints and tests
test-local: lint test

# Update dependencies
up:
    nix flake update
    cargo upgrade -i

# Run all lints
lint: fmt-check clippy clippy-all

# Check formatting
fmt-check:
    cargo fmt --all -- --check

# Clippy
clippy:
    cargo clippy -- -D warnings

# Clippy with all features
clippy-all:
    # Allow deprecated because we test the no_std feature.
    cargo clippy --all-features -- -D warnings -A deprecated

# Run test
test *args:
    cargo test {{args}}
