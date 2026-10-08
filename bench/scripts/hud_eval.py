"""Axis 3 (speed) of the first full hud evaluation, computed from pilot/<candidate>/speed/ and nothing else.

  hud_eval.py speed        writes pilot/hud_speed.json and pilot/hud_speed.md

S1: hud median at most 0.10x Python Rich. S2 to S4: hud median at most 0.80x the best existing Rust candidate whose
output is verified equal for that workload, with the CI upper bound at most 1.00, and no workload more than 25% slower
than that candidate (point estimate and CI upper bound both at most 1.25). Thresholds are the ones of evaluation.md;
none is changed. The 'best existing' set excludes hud itself.
"""

import json
import statistics
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from common import BENCH  # noqa: E402
from speed import ratio_ci  # noqa: E402

P = BENCH / "pilot"
EXISTING = ["rich_rust", "rs_rich", "richrs", "rich_rs", "composed"]
KEY = {"S1": "first_byte_ns", "S2": "ns", "S3": "ns", "S4": "ns"}


def load(name, workload):
    path = P / name / "speed" / f"{workload}.json"
    if not path.exists():
        return None
    return json.loads(path.read_text())[KEY[workload]]


def med(xs):
    return statistics.median(xs)


def main():
    rows, out = [], {}
    head = ["| workload | hud median ms | Python Rich ms | hud vs Python (CI) | best verified existing | hud vs best (CI) | threshold | result |",
            "|---|---|---|---|---|---|---|---|"]
    for w in ("S1", "S2", "S3", "S4"):
        hud, py = load("hud", w), load("python", w)
        existing = {n: s for n in EXISTING if (s := load(n, w))}
        best = min(existing, key=lambda n: med(existing[n])) if existing else None
        r_py = ratio_ci(hud, py)
        entry = {"hud_median_ms": med(hud) / 1e6, "python_median_ms": med(py) / 1e6, "ratio_python": r_py[0], "ci_python": [r_py[1], r_py[2]],
                 "best_existing": best, "verified_existing": sorted(existing)}
        if w == "S1":
            r_b = ratio_ci(hud, existing[best])
            ok = r_py[0] <= 0.10
            regress = r_b[0] <= 1.25 and r_b[2] <= 1.25
            entry.update(threshold="median <= 0.10x Python Rich; no regression above 25% against the best existing", point_ok=ok,
                         ci_ok=r_py[2] <= 0.10, ratio_best=r_b[0], ci_best=[r_b[1], r_b[2]], best_median_ms=med(existing[best]) / 1e6,
                         no_regress_ok=regress, gate=ok and regress)
            rows.append(f"| S1 | {med(hud) / 1e6:.3f} | {med(py) / 1e6:.2f} | {r_py[0]:.3f} [{r_py[1]:.3f}, {r_py[2]:.3f}] | "
                        f"{best} {med(existing[best]) / 1e6:.2f} ms | {r_b[0]:.3f} [{r_b[1]:.3f}, {r_b[2]:.3f}] | <= 0.10x Python, no regression > 25% | {'pass' if ok and regress else 'FAIL'} |")
        else:
            r_b = ratio_ci(hud, existing[best])
            point = r_b[0] <= 0.80
            ci = r_b[2] <= 1.00
            regress = r_b[0] <= 1.25 and r_b[2] <= 1.25
            gate = point and ci and regress
            entry.update(ratio_best=r_b[0], ci_best=[r_b[1], r_b[2]], best_median_ms=med(existing[best]) / 1e6,
                         point_ok=point, ci_ok=ci, no_regress_ok=regress, gate=gate,
                         threshold="median <= 0.80x best, CI upper <= 1.00, no regression above 25%")
            rows.append(f"| {w} | {med(hud) / 1e6:.3f} | {med(py) / 1e6:.2f} | {r_py[0]:.3f} [{r_py[1]:.3f}, {r_py[2]:.3f}] | "
                        f"{best} {med(existing[best]) / 1e6:.2f} ms | {r_b[0]:.3f} [{r_b[1]:.3f}, {r_b[2]:.3f}] | <= 0.80x best, CI <= 1.00 | {'pass' if gate else 'FAIL'} |")
        out[w] = entry
    out["all_pass"] = all(out[w]["gate"] for w in ("S1", "S2", "S3", "S4"))
    (P / "hud_speed.json").write_text(json.dumps(out, indent=1))
    (P / "hud_speed.md").write_text("\n".join(head + rows) + "\n")
    print("\n".join(head + rows))
    print("all speed thresholds hold:", out["all_pass"])


if __name__ == "__main__":
    main()
