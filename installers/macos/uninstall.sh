#!/usr/bin/env sh
PREFIX="${1:-$HOME/.local}"
rm -f "$PREFIX/bin/rmojo"; rm -rf "$PREFIX/share/rmojo"; echo "rmojo removed from $PREFIX"
