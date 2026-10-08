#!/usr/bin/env bash
# Sequential speed run for rs-rich and the Python Rich reference under identical conditions (bench-design.md rule 1).
set -u
BENCH="$(cd "$(dirname "$0")/../../.." && pwd)"
OUT="$BENCH/candidates/rs_rich/results/speed"
RS="$BENCH/candidates/rs_rich/target/release"
PY="$BENCH/.venv/bin/python"
mkdir -p "$OUT"
busy() { pgrep -x 'cargo|rustc|cc|ld|rustdoc|clang' | tr '\n' ' '; }
{
  echo "machine: $(sysctl -n machdep.cpu.brand_string), $(sysctl -n hw.memsize) bytes, macOS $(sw_vers -productVersion)"
  echo "rustc: $(rustc -V); cargo: $(cargo -V)"
  echo "start: $(date -u +%FT%TZ) load: $(uptime | sed 's/.*load/load/')"
} > "$OUT/conditions.txt"
cd "$BENCH"
for w in ${WORKLOADS:-S1 S2 S3 S4}; do
  for who in rs_rich python; do
    if [ "$who" = rs_rich ]; then
      [ "$w" = S1 ] && cmd=("$RS/s1") || cmd=("$RS/bench")
    else
      [ "$w" = S1 ] && cmd=("$PY" "$BENCH/reference/python/speed/s1.py") || cmd=("$PY" "$BENCH/reference/python/speed/bench.py")
    fi
    for attempt in 1 2 3 4 5 6 7 8 9 10; do
      until [ -z "$(busy)" ]; do sleep 2; done
      echo "[$(date -u +%T)] $w $who attempt=$attempt busy-before=[$(busy)] load=$(uptime | sed 's/.*load averages: //')" >> "$OUT/conditions.txt"
      uv run python scripts/speed.py time "$w" --iterations 30 --out "$OUT/${w}_${who}.json" -- "${cmd[@]}" > "$OUT/${w}_${who}.txt" 2>&1
      after="$(busy)"
      echo "[$(date -u +%T)] $w $who done busy-after=[$after]" >> "$OUT/conditions.txt"
      [ -z "$after" ] && break
      echo "[$(date -u +%T)] $w $who CONTAMINATED, rerun" >> "$OUT/conditions.txt"
    done
  done
done
echo "end: $(date -u +%FT%TZ)" >> "$OUT/conditions.txt"
