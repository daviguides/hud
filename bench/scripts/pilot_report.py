"""Cross-candidate tables for PILOT-RESULTS.md, computed from pilot/<candidate>/ and nothing else.

  pilot_report.py            writes pilot/tables.md and pilot/summary.json

Thresholds are the ones of evaluation.md, copied here only to colour the cells; none is changed.
"""

import json
import statistics
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from common import BENCH  # noqa: E402
from speed import ratio_ci  # noqa: E402

P = BENCH / "pilot"
NAMES = ["rich_rust", "rs_rich", "richrs", "rich_rs", "composed"]
LABEL = {"rich_rust": "rich_rust 0.2.3", "rs_rich": "rs-rich 0.0.9", "richrs": "richrs 0.2.1", "rich_rs": "rich-rs 1.3.0",
         "composed": "composed (owo-colors + comfy-table + indicatif)"}


def jl(path):
    p = Path(path)
    return json.loads(p.read_text()) if p.exists() else None


def pct(a, b):
    return f"{a}/{b} = {a / b:.1%}" if b else "n/a"


def mark(ok):
    return "pass" if ok else "FAIL"


def load_all():
    d = {}
    for n in NAMES:
        x = {"corr": jl(P / n / "correctness.json"), "width": jl(P / n / "width.json"), "cap": jl(P / n / "capability.json"),
             "tasks": jl(P / n / "tasks.json"), "api": jl(P / n / "api.json")}
        x["docs"] = [jl(p) for p in sorted((P / n).glob("docs*.json"))]
        x["adopt"] = {p.stem.replace("adoption_", ""): jl(p) for p in sorted((P / n).glob("adoption_*.json"))}
        x["speed"] = {w: jl(P / n / "speed" / f"{w}.json") for w in ("S1", "S2", "S3", "S4")}
        x["speed_unv"] = {w: jl(P / n / "speed" / f"{w}.unverified.json") for w in ("S1", "S2", "S3", "S4")}
        x["speed_txt"] = {w: (P / n / "speed" / f"{w}.txt").read_text() if (P / n / "speed" / f"{w}.txt").exists() else ""
                          for w in ("S1", "S2", "S3", "S4")}
        d[n] = x
    d["python"] = {"speed": {w: jl(P / "python" / "speed" / f"{w}.json") for w in ("S1", "S2", "S3", "S4")}}
    return d


def correctness(d):
    rows = ["| candidate | style | markup | table | panel | tree | progress | error | overall | style+markup 100% | gate 98% |", "|---|---|---|---|---|---|---|---|---|---|---|"]
    out = {}
    for n in NAMES:
        c = d[n]["corr"]
        f = c["features"]
        cell = lambda k: f"{f[k]['pass']}/{f[k]['denominator']}" if k in f else "n/a"  # noqa: E731
        o = c["overall"]
        gate = o["rate"] is not None and o["rate"] >= 0.98
        out[n] = {"pass": o["pass"], "den": o["denominator"], "rate": o["rate"], "style_markup_100": o["style_markup_100"], "gate": gate}
        rows.append(f"| {LABEL[n]} | {cell('style')} | {cell('markup')} | {cell('table')} | {cell('panel')} | {cell('tree')} | {cell('progress')} | {cell('error')} | {pct(o['pass'], o['denominator'])} | {mark(o['style_markup_100'])} | {mark(gate)} |")
    return "\n".join(rows), out


def assertiveness(d):
    rows = ["| candidate | width agreement (gate 99%) | grapheme splits fold / truncate (gate 0) | misaligned table rows (gate 0) | capability (gate 40/40) |", "|---|---|---|---|---|"]
    out = {}
    for n in NAMES:
        w, cap = d[n]["width"], d[n]["cap"]
        wd, fo, tr, tb = w["width"], w["fold"], w["truncate"], w["tables"]
        fold = fo["grapheme_splits"] if fo.get("supported") else None
        trunc = tr["grapheme_splits"] if tr.get("supported") else None
        mis, tot = tb["misaligned_rows"], tb["rows"]
        capok = sum(1 for c in cap if c["ok"])
        out[n] = {"width_rate": wd["rate"], "fold_splits": fold, "truncate_splits": trunc, "misaligned": mis, "cap": f"{capok}/{len(cap)}",
                  "gate_width": wd["rate"] >= 0.99, "gate_splits": fold == 0 and trunc == 0,
                  "gate_tables": mis == 0 and tb["missing"] == 0, "gate_cap": capok == len(cap)}
        rows.append(f"| {LABEL[n]} | {pct(wd['agree'], wd['total'])} ({mark(wd['rate'] >= 0.99)}) | {fold if fold is not None else 'unsupported'} / {trunc if trunc is not None else 'unsupported'} ({mark(out[n]['gate_splits'])}) | {mis}/{tot} rows{', ' + str(tb['missing']) + ' of 12 tables unsupported' if tb['missing'] else ''} ({mark(out[n]['gate_tables'])}) | {capok}/{len(cap)} ({mark(out[n]['gate_cap'])}) |")
    return "\n".join(rows), out


def med(samples):
    return statistics.median(samples)


def speed(d):
    out = {"S1": {}, "S2": {}, "S3": {}, "S4": {}}
    lines = []
    py = d["python"]["speed"]
    verified = lambda n, w: d[n]["speed"][w] is not None  # noqa: E731
    head = ["| workload | candidate | verified | median ms | vs Python Rich (CI) | vs best verified Rust candidate (CI) |", "|---|---|---|---|---|---|"]
    for w in ("S1", "S2", "S3", "S4"):
        key = "first_byte_ns" if w == "S1" else "ns"
        py_s = py[w][key] if py[w] else None
        samples = {n: (d[n]["speed"][w][key] if verified(n, w) else None) for n in NAMES}
        ok = {n: s for n, s in samples.items() if s}
        best = min(ok, key=lambda n: med(ok[n])) if ok else None
        if py_s:
            lines.append(f"| {w} | Python Rich 15.0.0 | reference | {med(py_s) / 1e6:.2f} | 1 | n/a |")
        for n in NAMES:
            s = samples[n]
            if not s:
                u = d[n]["speed_unv"][w]
                if u and py_s:
                    us = u[key]
                    r, lo, hi = ratio_ci(us, py_s)
                    lines.append(f"| {w} | {LABEL[n]} | NO: output differs from the golden, workload discarded; informational timing of different work | {med(us) / 1e6:.2f} | {r:.3f} [{lo:.3f}, {hi:.3f}] (not comparable) | not ranked |")
                    out[w][n] = {"verified": False, "informational_median_ms": med(us) / 1e6, "ratio_py": r, "ci_py": [lo, hi]}
                else:
                    lines.append(f"| {w} | {LABEL[n]} | NO: output differs from the golden, workload discarded | n/a | n/a | n/a |")
                    out[w][n] = None
                continue
            r, lo, hi = ratio_ci(s, py_s) if py_s else (None, None, None)
            rb = ratio_ci(s, ok[best]) if best else (None,) * 3
            out[w][n] = {"verified": True, "median_ms": med(s) / 1e6, "ratio_py": r, "ci_py": [lo, hi], "ratio_best": rb[0], "ci_best": [rb[1], rb[2]], "best": best == n}
            lines.append(f"| {w} | {LABEL[n]} | yes | {med(s) / 1e6:.2f} | {r:.3f} [{lo:.3f}, {hi:.3f}] | {'(best)' if best == n else f'{rb[0]:.2f} [{rb[1]:.2f}, {rb[2]:.2f}]'} |")
    return "\n".join(head + lines), out


def dx(d):
    rows = ["| candidate | tasks passing (runs) | LOC median (passing) / ratio vs Python 12 | name parity (existence by name) | None-padding public fns | first-try |", "|---|---|---|---|---|---|"]
    out = {}
    for n in NAMES:
        t, a = d[n]["tasks"], d[n]["api"]
        ratio = t["loc_ratio_vs_python"]
        out[n] = {"tasks": t["tasks_passing"], "runs": f"{t['runs_passing']}/{t['runs_total']}", "loc_median": t["loc_median_passing"],
                  "loc_ratio": ratio, "parity": a["parity"], "none_padding": a["friction"]["none_padding"],
                  "gate_loc": ratio is not None and ratio <= 1.5, "gate_parity": a["parity"] >= 0.70, "gate_padding": a["friction"]["none_padding"] == 0}
        rows.append(f"| {LABEL[n]} | {t['tasks_passing']}/8 ({t['runs_passing']}/{t['runs_total']} runs) | {t['loc_median_passing']} / {ratio:.2f}x ({mark(out[n]['gate_loc'])}) | {a['parity_hit']}/{a['parity_total']} = {a['parity']:.1%} ({mark(out[n]['gate_parity'])}) | {a['friction']['none_padding']} ({mark(out[n]['gate_padding'])}) | not measured (agent runner not built) |")
    return "\n".join(rows), out


def adoption(d):
    rows = ["| candidate | project | added compile s (gate 15) | added stripped bytes (gate 1 500 000) | transitive deps (gate 60) | pass |", "|---|---|---|---|---|---|"]
    out = {}
    for n in NAMES:
        for k, a in d[n]["adopt"].items():
            if a is None:
                continue
            rows.append(f"| {LABEL[n]} | {k} | {a['added_compile_s']:.2f} | {a['added_binary_bytes']:,} | {a['transitive_deps']} | {mark(a['pass'])} |")
            out.setdefault(n, {})[k] = {"compile_s": a["added_compile_s"], "bytes": a["added_binary_bytes"], "deps": a["transitive_deps"], "pass": a["pass"]}
    return "\n".join(rows), out


def docs(d):
    rows = ["| candidate | crate | documented items | with an example | doctests (passed / failed / ignored) |", "|---|---|---|---|---|"]
    out = {}
    for n in NAMES:
        for x in d[n]["docs"]:
            dt = x.get("doctests", {})
            rows.append(f"| {LABEL[n]} | {x['crate']} | {x['documented']} items = {x['documented_pct']}% | {x['with_examples']} = {x['with_examples_pct']}% | {dt.get('passed', '?')} / {dt.get('failed', '?')} / {dt.get('ignored', '?')} |")
            out.setdefault(n, []).append({"crate": x["crate"], "documented_pct": x["documented_pct"], "examples_pct": x["with_examples_pct"], "doctests": dt})
    return "\n".join(rows), out


def main():
    d = load_all()
    parts, summary = {}, {}
    for name, fn in (("correctness", correctness), ("assertiveness", assertiveness), ("speed", speed), ("dx", dx), ("adoption", adoption), ("docs", docs)):
        parts[name], summary[name] = fn(d)
    md = "\n\n".join(f"### {k}\n\n{v}" for k, v in parts.items())
    (P / "tables.md").write_text(md + "\n")
    (P / "summary.json").write_text(json.dumps(summary, indent=1) + "\n")
    print(md)


if __name__ == "__main__":
    main()
