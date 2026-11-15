#!/usr/bin/env bash
export PATH="$HOME/.cargo/bin:/usr/bin:/bin"
cd "$(dirname "$0")"
export CARGO_TARGET_DIR="$HOME/rmojo-target"
for t in x86_64-unknown-freebsd x86_64-unknown-netbsd x86_64-unknown-illumos; do
  rustup target add $t >/dev/null 2>&1
  echo "== $t"; cargo check --release --target $t 2>&1 | tail -2
done
