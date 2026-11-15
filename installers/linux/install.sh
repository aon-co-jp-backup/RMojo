#!/usr/bin/env sh
# RMojo installer for Linux (x86_64 / aarch64). Usage: ./install.sh [--prefix DIR]   (default: ~/.local, no root needed)
set -eu
PREFIX="${HOME}/.local"
[ "${1:-}" = "--prefix" ] && PREFIX="$2"
HERE="$(cd "$(dirname "$0")" && pwd)"
mkdir -p "$PREFIX/bin" "$PREFIX/share/rmojo/examples"
install -m 755 "$HERE/rmojo" "$PREFIX/bin/rmojo"
[ -d "$HERE/examples" ] && cp -r "$HERE/examples/." "$PREFIX/share/rmojo/examples/"
echo "Installed rmojo to $PREFIX/bin  (add it to PATH if needed:  export PATH=\"$PREFIX/bin:\$PATH\")"
"$PREFIX/bin/rmojo" --version
