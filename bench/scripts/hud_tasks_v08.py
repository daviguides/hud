"""The three tasks added for v0.8 (t09 structured output, t10 composition, t11 width mix), for hud only.

  hud_tasks_v08.py [--out DIR]   checks every run against its golden, counts LOC with loc.py (the same rule as
                                 t01 to t08) and writes pilot/hud/tasks_v08.json

`pilot.py static hud` keeps checking t01 to t08 exactly as before; the new tasks have no solution in any other arm
(changelog 44), so they live in this script and the shared driver is untouched. Python Rich's LOC for the new tasks
is counted from its reference programs with the same rule.
"""

import argparse
import json
import re
import statistics
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import loc  # noqa: E402
from common import BENCH  # noqa: E402

PY = str(BENCH / ".venv" / "bin" / "python")
TASKS = {
    "t09-structured": "t09_structured",
    "t10-composition": "t10_composition",
    "t11-width-mix": "t11_width_mix",
}
REFS = {
    "t09-structured": "reference/python/tasks/t09_structured.py",
    "t10-composition": "reference/python/tasks/t10_composition.py",
    "t11-width-mix": "reference/python/tasks/t11_width_mix.py",
}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=str(BENCH / "pilot" / "hud"))
    args = ap.parse_args()
    runs, locs, py_locs = {}, {}, {}
    for tid, bin_ in TASKS.items():
        r = subprocess.run([PY, "scripts/tasks.py", "check", tid, "--", str(BENCH / "candidates/hud/target/release" / bin_)],
                           cwd=BENCH, capture_output=True, text=True)
        found = re.findall(r"^(PASS|FAIL) " + re.escape(tid) + r"/(\S+)", r.stdout, re.M)
        runs[tid] = {name: status == "PASS" for status, name in found}
        locs[tid] = loc.count(BENCH / "candidates/hud/src/bin" / f"{bin_}.rs")
        py_locs[tid] = loc.count(BENCH / REFS[tid])
    passing = [t for t in TASKS if runs[t] and all(runs[t].values())]
    result = {
        "runs": runs, "tasks_passing": len(passing), "tasks": len(TASKS),
        "runs_passing": sum(sum(v.values()) for v in runs.values()), "runs_total": sum(len(v) for v in runs.values()),
        "loc": locs, "python_loc": py_locs,
        "loc_median_passing": statistics.median([locs[t] for t in passing]) if passing else None,
        "python_loc_median": statistics.median(py_locs.values()),
    }
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    (out / "tasks_v08.json").write_text(json.dumps(result, indent=1) + "\n")
    print(json.dumps(result))


if __name__ == "__main__":
    main()
