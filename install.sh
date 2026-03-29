#!/usr/bin/env bash
# install.sh — Install scale-bridge as a standard Unix utility
#
# Usage:
#   ./install.sh                  # install to /usr/local (default)
#   PREFIX=/opt/local ./install.sh
#   ./install.sh --uninstall
#
# Installs:
#   $PREFIX/bin/scale-bridge
#   $PREFIX/share/man/man1/scale-bridge.1
#   $PREFIX/share/bash-completion/completions/scale-bridge.bash
#   $PREFIX/share/zsh/site-functions/_scale-bridge
#   $PREFIX/share/fish/vendor_completions.d/scale-bridge.fish

set -euo pipefail

PREFIX="${PREFIX:-/usr/local}"
BINARY_NAME="scale-bridge"
CRATE="scale-bridge-cli"

BIN_DIR="$PREFIX/bin"
MAN_DIR="$PREFIX/share/man/man1"
BASH_COMP_DIR="$PREFIX/share/bash-completion/completions"
ZSH_COMP_DIR="$PREFIX/share/zsh/site-functions"
FISH_COMP_DIR="$PREFIX/share/fish/vendor_completions.d"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BUILD_DIR="$SCRIPT_DIR/target/release"
GEN_DIR="$SCRIPT_DIR/target/generated"

# ── helpers ───────────────────────────────────────────────────────────────────

need() {
    command -v "$1" >/dev/null 2>&1 || { echo "error: '$1' is required but not found"; exit 1; }
}

info()  { echo "  [+] $*"; }
warn()  { echo "  [!] $*"; }

install_file() {
    local src="$1" dst_dir="$2" dst_name="${3:-$(basename "$1")}"
    mkdir -p "$dst_dir"
    install -m 644 "$src" "$dst_dir/$dst_name"
    info "installed $dst_dir/$dst_name"
}

install_bin() {
    local src="$1" dst_dir="$2"
    mkdir -p "$dst_dir"
    install -m 755 "$src" "$dst_dir/"
    info "installed $dst_dir/$(basename "$src")"
}

# ── uninstall ─────────────────────────────────────────────────────────────────

if [[ "${1:-}" == "--uninstall" ]]; then
    echo "Uninstalling $BINARY_NAME from $PREFIX …"
    rm -f "$BIN_DIR/$BINARY_NAME"
    rm -f "$MAN_DIR/$BINARY_NAME.1"
    rm -f "$BASH_COMP_DIR/$BINARY_NAME"
    rm -f "$ZSH_COMP_DIR/_$BINARY_NAME"
    rm -f "$FISH_COMP_DIR/$BINARY_NAME.fish"
    echo "Done."
    exit 0
fi

# ── build ─────────────────────────────────────────────────────────────────────

need cargo

cd "$SCRIPT_DIR"

echo "Building $BINARY_NAME (release) …"
cargo build --release -p "$CRATE"
info "binary: $BUILD_DIR/$BINARY_NAME"

echo "Generating man page and shell completions …"
mkdir -p "$GEN_DIR"
"$BUILD_DIR/scale-bridge-generate" "$GEN_DIR"

# ── install ───────────────────────────────────────────────────────────────────

echo "Installing to $PREFIX …"

install_bin "$BUILD_DIR/$BINARY_NAME" "$BIN_DIR"
install_file "$GEN_DIR/man/$BINARY_NAME.1" "$MAN_DIR"

install_file "$GEN_DIR/completions/$BINARY_NAME.bash" "$BASH_COMP_DIR" "$BINARY_NAME"
install_file "$GEN_DIR/completions/_$BINARY_NAME"     "$ZSH_COMP_DIR"  "_$BINARY_NAME"
install_file "$GEN_DIR/completions/$BINARY_NAME.fish" "$FISH_COMP_DIR" "$BINARY_NAME.fish"

# Update man database if mandb/makewhatis is available
if command -v mandb >/dev/null 2>&1; then
    mandb -q 2>/dev/null || true
elif command -v makewhatis >/dev/null 2>&1; then
    makewhatis "$MAN_DIR" 2>/dev/null || true
fi

echo ""
echo "scale-bridge installed successfully."
echo "  Run:  scale-bridge --help"
echo "  Man:  man scale-bridge"
