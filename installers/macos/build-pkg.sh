#!/usr/bin/env sh
# Build RMojo-<arch>.pkg (installs /usr/local/bin/rmojo).  usage: build-pkg.sh <rmojo-binary> <x64|arm64> <outdir>
set -eu
BIN="$1"; ARCH="$2"; OUT="${3:-.}"
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
VER="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -1)"
W="$(mktemp -d)"
mkdir -p "$W/root/usr/local/bin" "$W/root/usr/local/share/rmojo/examples"
install -m 755 "$BIN" "$W/root/usr/local/bin/rmojo"
cp -r "$ROOT/examples/." "$W/root/usr/local/share/rmojo/examples/"
mkdir -p "$OUT"
pkgbuild --root "$W/root" --identifier jp.co.aon.rmojo --version "$VER" --install-location / "$OUT/RMojo-$ARCH.pkg" >/dev/null
echo "$OUT/RMojo-$ARCH.pkg"
