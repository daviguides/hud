"""Show want/got for correctness case ids: python3 diffcase.py id [id ...]"""
import json
import sys
from pathlib import Path

BENCH = Path(__file__).resolve().parents[2]
cases = {}
for line in (BENCH / "cases/correctness.jsonl").read_text().splitlines():
    c = json.loads(line)
    cases[c["id"]] = c
for i in sys.argv[1:]:
    c = cases[i]
    want = (BENCH / f"golden/correctness/{i}.ansi").read_bytes().decode()
    got = (BENCH / f"results/rich_rust/correctness/{i}.ansi").read_bytes().decode()
    print("==", i, c["width"], c["color_system"], json.dumps(c["renderable"], ensure_ascii=False)[:400])
    wl, gl = want.split("\n"), got.split("\n")
    for k in range(max(len(wl), len(gl))):
        a = wl[k] if k < len(wl) else "<none>"
        b = gl[k] if k < len(gl) else "<none>"
        mark = "  " if a == b else "!!"
        print(f"{mark} W {a!r}")
        if a != b:
            print(f"{mark} G {b!r}")
