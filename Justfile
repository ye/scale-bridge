# scale-bridge Justfile
# Install just: cargo install just
# Install cargo-nextest: cargo install cargo-nextest

# Run all tests across the workspace
test:
    if cargo nextest --version >/dev/null 2>&1; then cargo nextest run --workspace; else cargo test --workspace; fi

# Run tests for a specific crate (e.g. just test-crate scale-bridge-scp01)
test-crate crate:
    if cargo nextest --version >/dev/null 2>&1; then cargo nextest run -p {{crate}}; else cargo test -p {{crate}}; fi

# Lint with clippy — deny all warnings
lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Check formatting (does not modify files)
fmt:
    cargo fmt --all --check

# Fix formatting in place
fmt-fix:
    cargo fmt --all

# Full CI check: fmt → lint → test (mirrors GitHub Actions)
ci: fmt lint test

# Build all crates in debug mode
build:
    cargo build --workspace

# Build release binary
release:
    cargo build --workspace --release

# Run CLI in mock mode
mock *ARGS:
    SCALE_BRIDGE_MOCK=1 cargo run -p scale-bridge-cli -- {{ARGS}}

# Generate and open documentation
docs:
    cargo doc --workspace --no-deps --open

# Show release binary size
size: release
    ls -lh target/release/scale-bridge

# Run fuzz tests only
fuzz:
    if cargo nextest --version >/dev/null 2>&1; then cargo nextest run -p scale-bridge-scp01 --test fuzz; else cargo test -p scale-bridge-scp01 --test fuzz; fi

# Generate man page and shell completions into target/generated/
generate: release
    mkdir -p target/generated
    ./target/release/scale-bridge-generate target/generated
    @echo "Man page:    target/generated/man/scale-bridge.1"
    @echo "Completions: target/generated/completions/"

# Install to /usr/local (or set PREFIX= to override)
install:
    ./install.sh

# Uninstall from /usr/local (or set PREFIX= to override)
uninstall:
    ./install.sh --uninstall

# Preview the man page without installing
man: generate
    man target/generated/man/scale-bridge.1

# Clean build artifacts
clean:
    cargo clean
