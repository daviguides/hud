"""How many corpus goldens depend on render order? Render every case with Rich in a fresh process, compare with golden/.

Run with bench/.venv python. Read-only on golden/. Output: results/golden_isolation_audit.json
"""
import json
import subprocess
import sys
from pathlib import Path

BENCH = Path(__file__).resolve().parents[3]
PY = BENCH / ".venv" / "bin" / "python"
SNIPPET = ("import sys, json; sys.path.insert(0, %r); import render_reference as r; "
           "sys.stdout.buffer.write(r.render_case(json.loads(sys.stdin.read())))" % str(BENCH / "scripts"))
rows = []
for corpus, gdir in (("correctness.jsonl", "correctness"), ("table_unicode.jsonl", "table_unicode")):
    for line in (BENCH / "cases" / corpus).read_text().splitlines():
        if not line.strip():
            continue
        case = json.loads(line)
        fresh = subprocess.run([str(PY), "-c", SNIPPET], input=line.encode(), capture_output=True, check=True).stdout
        golden = (BENCH / "golden" / gdir / f"{case['id']}.ansi").read_bytes()
        rows.append({"id": case["id"], "feature": case["feature"], "color_system": case["color_system"],
                     "golden_equals_isolated": fresh == golden})
polluted = [r for r in rows if not r["golden_equals_isolated"]]
by_feature = {}
for r in polluted:
    by_feature[r["feature"]] = by_feature.get(r["feature"], 0) + 1
out = {"cases": len(rows), "golden_differs_from_isolated_rich": len(polluted), "by_feature": by_feature,
       "ids": [r["id"] for r in polluted]}
Path(__file__).resolve().parents[1].joinpath("results", "golden_isolation_audit.json").write_text(json.dumps(out, indent=1) + "\n")
print({k: v for k, v in out.items() if k != "ids"})
