#!/usr/bin/env sh
# RMojo for Android (aarch64): run from Termux, or push with adb:
#   adb push rmojo /data/local/tmp/ && adb shell chmod 755 /data/local/tmp/rmojo && adb shell /data/local/tmp/rmojo --version
# In Termux:  ./install.sh   (installs to $PREFIX/bin)
set -eu
HERE="$(cd "$(dirname "$0")" && pwd)"
DEST="${PREFIX:-$HOME/.local}/bin"
mkdir -p "$DEST"
install -m 755 "$HERE/rmojo" "$DEST/rmojo"
echo "Installed rmojo to $DEST"; "$DEST/rmojo" --version
