"""Fuzz mode: zero panics on seeded random inputs, per feature.

A candidate ships a `fuzz` executable (`fuzz [--seed S] [--count N]`) that prints one JSON line per
feature: {"feature", "inputs", "panics", "violations", "first_panic", "first_violation"}, and exits
1 when a feature has a panic or a violated property. This driver runs it for each seed and applies
the criterion of evaluation.md (zero panics, at least 1 000 inputs per feature) to the totals.

  fuzz.py [--seeds 20261008 1 2] [--count 5000] [--out FILE] -- <candidate fuzz command>
"""

import argparse
import json
import subprocess
import sys

MIN_INPUTS = 1000


def main():
    argv = sys.argv[1:]
    cmd = []
    if "--" in argv:
        cut = argv.index("--")
        argv, cmd = argv[:cut], argv[cut + 1:]
    ap = argparse.ArgumentParser()
    ap.add_argument("--seeds", type=int, nargs="+", default=[20261008])
    ap.add_argument("--count", type=int, default=5000)
    ap.add_argument("--out")
    args = ap.parse_args(argv)
    totals = {}
    for seed in args.seeds:
        run = subprocess.run(cmd + ["--seed", str(seed), "--count", str(args.count)], capture_output=True, text=True)
        for line in run.stdout.splitlines():
            row = json.loads(line)
            t = totals.setdefault(row["feature"], {"inputs": 0, "panics": 0, "violations": 0, "first": None})
            t["inputs"] += row["inputs"]
            t["panics"] += row["panics"]
            t["violations"] += row["violations"]
            t["first"] = t["first"] or row.get("first_panic") or row.get("first_violation")
    ok = bool(totals) and all(t["panics"] == 0 and t["violations"] == 0 and t["inputs"] >= MIN_INPUTS for t in totals.values())
    for feature, t in sorted(totals.items()):
        print(f"{feature:<11} inputs {t['inputs']:>7}  panics {t['panics']}  violations {t['violations']}"
              + (f"  first: {t['first'][:160]}" if t["first"] else ""))
    print("FUZZ", "PASS" if ok else "FAIL", f"(seeds {args.seeds}, {args.count} inputs per feature per seed)")
    if args.out:
        with open(args.out, "w") as f:
            json.dump({"seeds": args.seeds, "count": args.count, "features": totals, "pass": ok}, f, indent=1)
            f.write("\n")
    raise SystemExit(0 if ok else 1)


if __name__ == "__main__":
    main()
