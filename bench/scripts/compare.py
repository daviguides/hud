"""Candidate-agnostic correctness comparison against the Rich goldens.

Candidate output directory:
  <id>.ansi          bytes the candidate wrote for case <id>
  <id>.unsupported   empty marker: the candidate has no support for this case
  support.json       {"features": ["style", "markup", ...]}  claimed features (default: all seen)
  classifications.json   {"<id>": {"class": "documented_deviation|reference_quirk|candidate_bug", "reason": "..."}}

Rules (evaluation.md, axis 1): rate = byte-identical / cases of claimed features after removing
documented_deviation and reference_quirk; unsupported is never a pass; style and markup must be 100%.
"""

import argparse
import json
import sys
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from common import BENCH, load_jsonl, read_bytes  # noqa: E402

EXCLUDED = {"documented_deviation", "reference_quirk"}
STRICT = ("style", "markup")


def first_diff(want: bytes, got: bytes):
    n = min(len(want), len(got))
    i = next((k for k in range(n) if want[k] != got[k]), n)
    line = want.count(b"\n", 0, i) + 1
    lo = max(0, i - 20)
    return {
        "byte_offset": i,
        "line": line,
        "want": want[lo:i + 40].decode("utf-8", "replace"),
        "got": got[lo:i + 40].decode("utf-8", "replace"),
        "want_len": len(want),
        "got_len": len(got),
    }


def compare(cand_dir, cases_path, golden_dir):
    cand = Path(cand_dir)
    cases = load_jsonl(cases_path)
    cls = json.loads((cand / "classifications.json").read_text()) if (cand / "classifications.json").exists() else {}
    support = json.loads((cand / "support.json").read_text()).get("features") if (cand / "support.json").exists() else None
    per = defaultdict(lambda: defaultdict(int))
    failures = []
    for c in cases:
        feat, cid = c["feature"], c["id"]
        golden = read_bytes(Path(golden_dir) / f"{cid}.ansi")
        out_path = cand / f"{cid}.ansi"
        if support is not None and feat not in support:
            per[feat]["not_claimed"] += 1
            continue
        if (cand / f"{cid}.unsupported").exists():
            per[feat]["unsupported"] += 1
            continue
        if not out_path.exists():
            per[feat]["missing"] += 1
            continue
        got = out_path.read_bytes()
        if got == golden:
            per[feat]["pass"] += 1
            continue
        klass = cls.get(cid, {}).get("class", "unclassified")
        if klass in EXCLUDED:
            per[feat]["excluded_" + klass] += 1
        else:
            per[feat]["fail"] += 1
        failures.append({"id": cid, "feature": feat, "class": klass, **first_diff(golden, got)})
    return per, failures


def summarize(per):
    rows, tp, tt = {}, 0, 0
    for feat, d in sorted(per.items()):
        denom = d["pass"] + d["fail"] + d["unsupported"] + d["missing"]
        rows[feat] = {**d, "denominator": denom, "rate": (d["pass"] / denom) if denom else None}
        if feat != "__none__" and denom:
            tp += d["pass"]
            tt += denom
    strict_ok = all(rows[f]["rate"] == 1.0 for f in STRICT if f in rows and rows[f]["denominator"])
    return rows, {"pass": tp, "denominator": tt, "rate": tp / tt if tt else None, "style_markup_100": strict_ok}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("candidate_dir", type=Path)
    ap.add_argument("--cases", type=Path, default=BENCH / "cases" / "correctness.jsonl")
    ap.add_argument("--golden", type=Path, default=BENCH / "golden" / "correctness")
    ap.add_argument("--show", type=int, default=5, help="print the first N failures")
    ap.add_argument("--out", type=Path)
    args = ap.parse_args()
    per, failures = compare(args.candidate_dir, args.cases, args.golden)
    rows, overall = summarize(per)
    for feat, r in rows.items():
        rate = "n/a" if r["rate"] is None else f"{r['rate']:.1%}"
        extra = {k: v for k, v in r.items() if k not in ("pass", "denominator", "rate") and v}
        print(f"{feat:9s} {r['pass']:3d}/{r['denominator']:3d} {rate:>6s} {extra if extra else ''}")
    rate = "n/a" if overall["rate"] is None else f"{overall['rate']:.1%}"
    print(f"overall   {overall['pass']}/{overall['denominator']} {rate}; style+markup 100%: {overall['style_markup_100']}")
    for f in failures[: args.show]:
        print(f"FAIL {f['id']} [{f['class']}] line {f['line']} offset {f['byte_offset']}:\n  want {f['want']!r}\n  got  {f['got']!r}")
    if args.out:
        args.out.write_text(json.dumps({"features": rows, "overall": overall, "failures": failures},
                                       indent=2, ensure_ascii=False) + "\n")


if __name__ == "__main__":
    main()
