#!/usr/bin/env sh
# RMojo installer for macOS (Intel / Apple silicon). Usage: ./install.sh [--prefix DIR]   (default: ~/.local, no admin needed)
set -eu
PREFIX="${HOME}/.local"
[ "${1:-}" = "--prefix" ] && PREFIX="$2"
HERE="$(cd "$(dirname "$0")" && pwd)"
mkdir -p "$PREFIX/bin" "$PREFIX/share/rmojo/examples"
install -m 755 "$HERE/rmojo" "$PREFIX/bin/rmojo"
xattr -d com.apple.quarantine "$PREFIX/bin/rmojo" 2>/dev/null || true   # downloaded, unsigned binary
[ -d "$HERE/examples" ] && cp -R "$HERE/examples/." "$PREFIX/share/rmojo/examples/"
echo "Installed rmojo to $PREFIX/bin  (add it to PATH if needed:  export PATH=\"$PREFIX/bin:\$PATH\")"
"$PREFIX/bin/rmojo" --version
