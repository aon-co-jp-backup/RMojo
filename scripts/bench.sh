#!/usr/bin/env bash
cd "$(dirname "$0")/.."
R="$HOME/rmojo-target/release/rmojo"
for f in "$@"; do
  s=$(date +%s.%N); python3 -c "exec(open('$f').read()); main()" >/dev/null; m=$(date +%s.%N)
  "$R" run "$f" >/dev/null; e=$(date +%s.%N)
  echo "$f cpython=$(echo "$m - $s" | bc)s rmojo=$(echo "$e - $m" | bc)s"
done
