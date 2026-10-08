"""Render every correctness case in a fresh Python process and compare with the stored golden.

Rich caches ANSI codes on Style objects (Style.parse is lru-cached and Style._ansi is set on first render), so
a render depends on which color depth the same style string was rendered at earlier in the process.
Usage: golden_pollution.py <out.json> <fresh_dir>   (run from bench/ with the pinned venv python)
The fresh render of every case that differs from its stored golden is written to <fresh_dir>/<id>.ansi.
"""
import json
import subprocess
import sys
from pathlib import Path

BENCH = Path(__file__).resolve().parents[2]
ONE = r"""
import json, sys
sys.path.insert(0, "scripts")
import render_reference as rr
case = json.loads(sys.argv[1])
sys.stdout.buffer.write(rr.render_case(case))
"""
bad = []
fresh_dir = Path(sys.argv[2])
fresh_dir.mkdir(parents=True, exist_ok=True)
for line in (BENCH / "cases/correctness.jsonl").read_text().splitlines():
    case = json.loads(line)
    fresh = subprocess.run([sys.executable, "-c", ONE, line], cwd=BENCH, capture_output=True).stdout
    gold = (BENCH / f"golden/correctness/{case['id']}.ansi").read_bytes()
    if fresh != gold:
        bad.append(case["id"])
        (fresh_dir / f"{case['id']}.ansi").write_bytes(fresh)
Path(sys.argv[1]).write_text(json.dumps({"differs_from_fresh": bad, "count": len(bad)}, indent=1))
print(len(bad), "of 210 goldens differ from a fresh-process render")
