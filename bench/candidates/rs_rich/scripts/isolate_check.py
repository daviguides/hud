"""Re-render each failing correctness case with Python Rich in a FRESH process and compare with the candidate.

Rich caches a Style's ANSI codes on first use regardless of color system, so a golden rendered after another
case that used the same style depends on case order. A failure where the candidate equals the isolated Rich output
is a `reference_quirk` (golden order dependence); any other failure stays unclassified for review.

  isolate_check.py <results/correctness.json> <candidate_outdir> <cases.jsonl>   (run with bench/.venv python)
"""
import json
import subprocess
import sys
from pathlib import Path

BENCH = Path(__file__).resolve().parents[3]
PY = BENCH / ".venv" / "bin" / "python"
SNIPPET = (
    "import sys, json; sys.path.insert(0, %r); import render_reference as r; "
    "case = json.loads(sys.stdin.read()); sys.stdout.buffer.write(r.render_case(case))" % str(BENCH / "scripts")
)


def main():
    report, outdir, cases_path = Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3])
    cases = {c["id"]: c for c in map(json.loads, filter(str.strip, cases_path.read_text().splitlines()))}
    failing = [f["id"] for f in json.loads(report.read_text())["failures"]]
    classifications, genuine = {}, []
    for cid in failing:
        fresh = subprocess.run([str(PY), "-c", SNIPPET], input=json.dumps(cases[cid]).encode(), capture_output=True,
                               check=True)
        want = fresh.stdout
        got = (outdir / f"{cid}.ansi").read_bytes()
        if got == want:
            classifications[cid] = {"class": "reference_quirk",
                                    "reason": "golden depends on render order (Rich caches a Style's ANSI codes on "
                                              "first use, whatever the color system); a fresh Rich process renders "
                                              "exactly the candidate's bytes"}
        else:
            genuine.append(cid)
    (outdir / "classifications.json").write_text(json.dumps(classifications, indent=1) + "\n")
    print(f"failing {len(failing)}; equal to isolated Rich (reference_quirk): {len(classifications)}; "
          f"genuine differences: {len(genuine)}")
    print("genuine:", genuine)


if __name__ == "__main__":
    main()
