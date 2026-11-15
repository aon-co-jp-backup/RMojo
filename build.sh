#!/usr/bin/env bash
export PATH="$HOME/.cargo/bin:/usr/bin:/bin"
cd "$(dirname "$0")"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/rmojo-target}"
cargo build --release 2>&1 | tail -5
python3 tests/run.py "$CARGO_TARGET_DIR/release/rmojo" 2>&1 | tail -8
