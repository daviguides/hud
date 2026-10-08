"""Classify the correctness failures of the rich_rust adapter.

  analyze_correctness.py <candidate_dir> <fresh_golden_dir> <out_dir>

Rules (evaluation.md, pilot validity 5; compare.py classes):
- reference_quirk: the stored golden differs from a fresh-process Rich render (Rich caches ANSI codes on Style
  objects, so a golden depends on the color depth rendered earlier in the same process) AND the candidate's
  bytes equal the fresh render. A candidate that also fails the fresh render stays a failure.
- candidate_bug: everything else; the reason says whether the visible text (ANSI stripped) is identical.
Writes <out_dir>/classifications.json (read by compare.py) and prints a summary.
"""
import json
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

BENCH = Path(__file__).resolve().parents[2]
ANSI = re.compile(rb"\x1b\[[0-9;]*m")
cand, fresh_dir, out = Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3])
cases = [json.loads(line) for line in (BENCH / "cases/correctness.jsonl").read_text().splitlines()]
cls, rows = {}, defaultdict(Counter)
scored_fresh = defaultdict(Counter)
for case in cases:
    i, feat = case["id"], case["feature"]
    stored = (BENCH / f"golden/correctness/{i}.ansi").read_bytes()
    got = (cand / f"{i}.ansi").read_bytes()
    fresh_path = fresh_dir / f"{i}.ansi"
    reference = fresh_path.read_bytes() if fresh_path.exists() else stored
    scored_fresh[feat]["pass" if got == reference else "fail"] += 1
    if got == stored:
        rows[feat]["pass"] += 1
        continue
    if fresh_path.exists() and got == fresh_path.read_bytes():
        cls[i] = {"class": "reference_quirk", "reason": "stored golden depends on Rich's process-wide style cache; a fresh-process render equals the candidate"}
        rows[feat]["reference_quirk"] += 1
        continue
    visible_equal = ANSI.sub(b"", got) == ANSI.sub(b"", reference)
    reason = "visible text identical to the reference; SGR sequences differ" if visible_equal else "visible text differs from the reference (layout)"
    if fresh_path.exists():
        reason += " (compared against the fresh-process render)"
    cls[i] = {"class": "candidate_bug", "reason": reason}
    rows[feat]["fail_visible_equal" if visible_equal else "fail_layout"] += 1
out.mkdir(parents=True, exist_ok=True)
(out / "classifications.json").write_text(json.dumps(cls, indent=1) + "\n")
print(f"{'feature':9s} {'pass':>4s} {'refquirk':>8s} {'visible=':>8s} {'layout':>6s} | vs fresh golden pass/total")
for feat in ["style", "markup", "table", "panel", "tree", "progress", "error"]:
    r, f = rows[feat], scored_fresh[feat]
    print(f"{feat:9s} {r['pass']:4d} {r['reference_quirk']:8d} {r['fail_visible_equal']:8d} {r['fail_layout']:6d} | {f['pass']}/{f['pass'] + f['fail']}")
