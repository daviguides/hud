#!/usr/bin/env bash
# adoption.py with a contamination guard: rerun until no other cargo/rustc/cc/ld process is seen before or after.
set -u
BENCH="$(cd "$(dirname "$0")/../../.." && pwd)"
project="$1"; out="$2"
busy() { pgrep -x 'cargo|rustc|cc|ld|rustdoc|clang' | tr '\n' ' '; }
cd "$BENCH"
for attempt in 1 2 3 4 5 6 7 8 9 10 11 12; do
  until [ -z "$(busy)" ]; do sleep 2; done
  echo "attempt=$attempt load=$(uptime | sed 's/.*averages: //')"
  uv run python scripts/adoption.py "$project" --out "$out" > "$out.txt"
  after="$(busy)"
  [ -z "$after" ] && { echo "clean run"; cat "$out.txt"; exit 0; }
  echo "contaminated (busy after: $after), rerun"
done
echo "gave up: machine never quiet"; exit 1
