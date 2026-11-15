#!/usr/bin/env bash
export PATH="$HOME/.cargo/bin:/usr/bin:/bin"
cd "$(dirname "$0")"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/rmojo-target}"
cargo test --release 2>&1 | grep -E "^test |test result|error" | head -20
cargo build --release 2>&1 | grep -E "^error" -A5
python3 tests/run.py "$CARGO_TARGET_DIR/release/rmojo" | tail -5
"$CARGO_TARGET_DIR/release/rmojo" run tests/mojo_struct.mojo
for f in tests/mojo_*.expected; do
  b="${f%.expected}"
  if diff <("$CARGO_TARGET_DIR/release/rmojo" run "$b.mojo" 2>&1) "$f" >/dev/null; then echo "OK   $b.mojo"; else echo "DIFF $b.mojo"; "$CARGO_TARGET_DIR/release/rmojo" run "$b.mojo" 2>&1 | head -20; fi
done
