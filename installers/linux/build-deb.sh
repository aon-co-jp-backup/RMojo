#!/usr/bin/env sh
# Build rmojo_<ver>_<arch>.deb.  usage: build-deb.sh <rmojo-binary> <amd64|arm64> <outdir>
set -eu
BIN="$1"; ARCH="$2"; OUT="${3:-.}"
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
VER="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -1)"
W="$(mktemp -d)"; P="$W/rmojo_${VER}_${ARCH}"
mkdir -p "$P/DEBIAN" "$P/usr/bin" "$P/usr/share/rmojo/examples" "$P/usr/share/doc/rmojo"
install -m 755 "$BIN" "$P/usr/bin/rmojo"
cp -r "$ROOT/examples/." "$P/usr/share/rmojo/examples/"
cp "$ROOT/README.md" "$ROOT/LICENSE" "$P/usr/share/doc/rmojo/"
SIZE="$(du -sk "$P/usr" | cut -f1)"
cat > "$P/DEBIAN/control" <<EOF
Package: rmojo
Version: $VER
Section: devel
Priority: optional
Architecture: $ARCH
Installed-Size: $SIZE
Maintainer: aon-co-jp <noreply@users.noreply.github.com>
Homepage: https://github.com/aon-co-jp/RMojo
Description: RMojo - a Mojo/Python-subset runtime written in Rust
 Single static binary. Runs .mojo and .py files; no C++ runtime needed.
 Updates come from new .deb releases (the self-updater cannot write to /usr/bin).
EOF
mkdir -p "$OUT"
dpkg-deb --root-owner-group --build "$P" "$OUT/rmojo_${VER}_${ARCH}.deb" >/dev/null
echo "$OUT/rmojo_${VER}_${ARCH}.deb"
