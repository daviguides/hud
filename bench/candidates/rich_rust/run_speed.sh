#!/bin/bash
# Sequential speed run for the rich_rust candidate. Usage: bash run_speed.sh (from bench/)
set -u
BENCH=$(cd "$(dirname "$0")/../.." && pwd)
cd "$BENCH"
B=$BENCH/candidates/rich_rust/adapters/target/release
OUT=$BENCH/results/rich_rust/speed
mkdir -p "$OUT"
printf '#!/bin/sh\nexec %s/bench "$@" --detect-size\n' "$B" > "$OUT/bench-detect"
printf '#!/bin/sh\nexec %s/bench "$@" --buffered\n' "$B" > "$OUT/bench-buffered"
printf '#!/bin/sh\nexec %s/s1 --explicit-size\n' "$B" > "$OUT/s1-explicit"
chmod +x "$OUT/bench-detect" "$OUT/bench-buffered" "$OUT/s1-explicit"
busy() { pgrep -x rustc >/dev/null || pgrep -x cargo >/dev/null; }
run() { # name workload cmd...
  local name=$1 w=$2; shift 2
  if busy; then echo "WARN rustc/cargo running before $name" >> "$OUT/run.log"; fi
  echo "== $name $w $(date +%T)" >> "$OUT/run.log"
  uv run python scripts/speed.py verify "$w" -- "$@" >> "$OUT/run.log" 2>&1
  uv run python scripts/speed.py time "$w" --out "$OUT/$name-$w.json" -- "$@" >> "$OUT/run.log" 2>&1
  if busy; then echo "WARN rustc/cargo running after $name" >> "$OUT/run.log"; fi
}
: > "$OUT/run.log"
run primary S1 "$B/s1"
run explicit S1 "$OUT/s1-explicit"
for w in S2 S3 S4; do run primary $w "$B/bench"; done
for w in S2 S4; do run buffered $w "$OUT/bench-buffered"; done
for w in S2 S4 S3; do run detect $w "$OUT/bench-detect"; done
run python S1 "$BENCH/.venv/bin/python" "$BENCH/reference/python/speed/s1.py"
for w in S2 S3 S4; do run python $w "$BENCH/.venv/bin/python" "$BENCH/reference/python/speed/bench.py"; done
echo "== done $(date +%T)" >> "$OUT/run.log"
