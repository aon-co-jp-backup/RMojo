#!/usr/bin/env bash
export PATH="$HOME/.cargo/bin:/usr/bin:/bin"
cd "$(dirname "$0")"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/rmojo-target}"
cargo test --release 2>&1 | grep -E "^test |test result|error" | head -20
cargo build --release 2>&1 | grep -E "^error" -A5
python3 tests/run.py "$CARGO_TARGET_DIR/release/rmojo" | tail -5
"$CARGO_TARGET_DIR/release/rmojo" run tests/mojo_fn.mojo
