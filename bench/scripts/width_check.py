"""Assertiveness checks: width agreement, grapheme-safe fold/truncate, table alignment.

Candidate output directory layout (all optional; missing = unsupported, never a pass):
  width.jsonl      {"id", "width"}                 one per corpus string
  fold.jsonl       {"id", "w", "lines": [...]}     w = 1..40, hard fold at grapheme boundaries
  truncate.jsonl   {"id", "w", "text"}             longest grapheme-boundary prefix of width <= w
  tables/<id>.ansi                                 output for cases/table_unicode.jsonl
  panels/<id>-expand.ansi, panels/<id>-fit.ansi    every Unicode table case inside a panel
  trees/<id>.ansi                                  every Unicode table case as tree labels
"""

import argparse
import json
import sys
from collections import Counter, defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from common import (  # noqa: E402
    BENCH, fold_ref, graphemes, load_jsonl, rich_width, strip_ansi, truncate_ref,
)

WIDTHS = range(1, 41)


def boundaries(text):
    out, pos = {0}, 0
    for g in graphemes(text):
        pos += len(g)
        out.add(pos)
    return out


def check_width(cand_dir):
    ref = {r["id"]: r for r in load_jsonl(BENCH / "golden" / "width" / "width_ref.jsonl")}
    path = Path(cand_dir) / "width.jsonl"
    if not path.exists():
        return {"supported": False}
    got = {r["id"]: r["width"] for r in load_jsonl(path)}
    agree, by_cat, wrong = 0, defaultdict(lambda: [0, 0]), []
    contested = [0, 0]
    for i, r in ref.items():
        ok = got.get(i) == r["ref"]
        by_cat[r["category"]][1] += 1
        by_cat[r["category"]][0] += ok
        agree += ok
        if r["status"] == "resolved":
            contested[1] += 1
            contested[0] += ok
        if not ok:
            wrong.append({"id": i, "want": r["ref"], "got": got.get(i), "category": r["category"]})
    return {
        "supported": True, "total": len(ref), "agree": agree, "rate": agree / len(ref),
        "contested_agree": contested[0], "contested_total": contested[1],
        "by_category": {k: {"agree": v[0], "total": v[1]} for k, v in sorted(by_cat.items())},
        "wrong": wrong[:50],
    }


def check_fold(cand_dir):
    path = Path(cand_dir) / "fold.jsonl"
    if not path.exists():
        return {"supported": False}
    texts = {r["id"]: r["text"] for r in load_jsonl(BENCH / "cases" / "width_corpus.jsonl")}
    bounds = {i: boundaries(t) for i, t in texts.items()}
    got = {(r["id"], r["w"]): r["lines"] for r in load_jsonl(path)}
    total = len(texts) * len(WIDTHS)
    split = concat_bad = too_wide = different = missing = 0
    examples = []
    for i, text in texts.items():
        for w in WIDTHS:
            lines = got.get((i, w))
            if lines is None:
                missing += 1
                continue
            if "".join(lines) != text:
                concat_bad += 1
                examples.append({"id": i, "w": w, "problem": "concat"})
                continue
            pos, bad = 0, False
            for ln in lines[:-1]:
                pos += len(ln)
                if pos not in bounds[i]:
                    bad = True
            if bad:
                split += 1
                examples.append({"id": i, "w": w, "problem": "grapheme split"})
                continue
            if any(rich_width(ln) > w and len(graphemes(ln)) > 1 for ln in lines):
                too_wide += 1
                continue
            if lines != fold_ref(text, w):
                different += 1
    return {
        "supported": True, "total": total, "missing": missing, "grapheme_splits": split,
        "concat_mismatch": concat_bad, "line_too_wide": too_wide, "valid_but_not_greedy": different,
        "examples": examples[:20],
    }


def check_truncate(cand_dir):
    path = Path(cand_dir) / "truncate.jsonl"
    if not path.exists():
        return {"supported": False}
    texts = {r["id"]: r["text"] for r in load_jsonl(BENCH / "cases" / "width_corpus.jsonl")}
    bounds = {i: boundaries(t) for i, t in texts.items()}
    got = {(r["id"], r["w"]): r["text"] for r in load_jsonl(path)}
    total = len(texts) * len(WIDTHS)
    split = not_prefix = different = missing = 0
    for i, text in texts.items():
        for w in WIDTHS:
            res = got.get((i, w))
            if res is None:
                missing += 1
            elif not text.startswith(res):
                not_prefix += 1
            elif len(res) not in bounds[i]:
                split += 1
            elif res != truncate_ref(text, w):
                different += 1
    return {"supported": True, "total": total, "missing": missing, "grapheme_splits": split,
            "not_a_prefix": not_prefix, "valid_but_not_maximal": different}


def table_misaligned(data: bytes):
    lines = strip_ansi(data).decode("utf-8", "replace").rstrip("\n").split("\n")
    widths = [rich_width(ln) for ln in lines]
    if not widths:
        return 0, 0
    common = Counter(widths).most_common(1)[0][0]
    return sum(1 for w in widths if w != common), len(widths)


def check_tables(cand_dir):
    cases = load_jsonl(BENCH / "cases" / "table_unicode.jsonl")
    base = Path(cand_dir) / "tables"
    if not base.exists():
        return {"supported": False}
    bad_rows = rows = missing = bad_tables = 0
    for c in cases:
        p = base / f"{c['id']}.ansi"
        if not p.exists():
            missing += 1
            continue
        bad, n = table_misaligned(p.read_bytes())
        bad_rows += bad
        rows += n
        bad_tables += bad > 0
    return {"supported": True, "tables": len(cases), "missing": missing, "misaligned_rows": bad_rows,
            "rows": rows, "tables_with_misalignment": bad_tables}


def check_widgets(cand_dir):
    """Panel and tree rows over the Unicode table cases: no row wider than the terminal; panel rows
    all as wide as the panel (the most common row width), and an expanding panel as wide as the
    terminal."""
    cases = {c["id"]: c for c in load_jsonl(BENCH / "cases" / "table_unicode.jsonl")}
    base = Path(cand_dir)
    if not (base / "panels").exists() and not (base / "trees").exists():
        return {"supported": False}
    rows = too_wide = misaligned = outputs = missing = 0
    for case_id, case in cases.items():
        terminal = case["width"]
        found = {"panels": [(f"{case_id}-expand", True), (f"{case_id}-fit", False)], "trees": [(case_id, None)]}
        for folder, items in found.items():
            for name, expand in items:
                path = base / folder / f"{name}.ansi"
                if not path.exists():
                    missing += 1
                    continue
                outputs += 1
                lines = strip_ansi(path.read_bytes()).decode("utf-8", "replace").rstrip("\n").split("\n")
                widths = [rich_width(ln) for ln in lines]
                rows += len(widths)
                too_wide += sum(1 for w in widths if w > terminal)
                if folder == "panels":
                    common = Counter(widths).most_common(1)[0][0]
                    misaligned += sum(1 for w in widths if w != common)
                    if expand and common != terminal:
                        misaligned += len(widths)
    return {"supported": True, "outputs": outputs, "missing": missing, "rows": rows,
            "rows_wider_than_terminal": too_wide, "panel_rows_misaligned": misaligned}


def run(cand_dir):
    return {"width": check_width(cand_dir), "fold": check_fold(cand_dir),
            "truncate": check_truncate(cand_dir), "tables": check_tables(cand_dir),
            "widgets": check_widgets(cand_dir)}


def write_reference(out):
    """Emit the Rich-based reference as a candidate output directory (harness self-test)."""
    out = Path(out)
    (out / "tables").mkdir(parents=True, exist_ok=True)
    items = load_jsonl(BENCH / "cases" / "width_corpus.jsonl")
    ref = {r["id"]: r["ref"] for r in load_jsonl(BENCH / "golden" / "width" / "width_ref.jsonl")}
    with (out / "width.jsonl").open("w") as f:
        for it in items:
            f.write(json.dumps({"id": it["id"], "width": ref[it["id"]]}) + "\n")
    with (out / "fold.jsonl").open("w") as f, (out / "truncate.jsonl").open("w") as g:
        for it in items:
            for w in WIDTHS:
                f.write(json.dumps({"id": it["id"], "w": w, "lines": fold_ref(it["text"], w)}, ensure_ascii=False) + "\n")
                g.write(json.dumps({"id": it["id"], "w": w, "text": truncate_ref(it["text"], w)}, ensure_ascii=False) + "\n")
    for c in load_jsonl(BENCH / "cases" / "table_unicode.jsonl"):
        (out / "tables" / f"{c['id']}.ansi").write_bytes((BENCH / "golden" / "table_unicode" / f"{c['id']}.ansi").read_bytes())


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("candidate_dir", type=Path)
    ap.add_argument("--write-reference", action="store_true", help="fill candidate_dir from the reference (self-test)")
    ap.add_argument("--out", type=Path)
    args = ap.parse_args()
    if args.write_reference:
        write_reference(args.candidate_dir)
    report = run(args.candidate_dir)
    text = json.dumps(report, indent=2, ensure_ascii=False)
    if args.out:
        args.out.write_text(text + "\n")
    w, f, t, tb = report["width"], report["fold"], report["truncate"], report["tables"]
    if w["supported"]:
        print(f"width: {w['agree']}/{w['total']} ({w['rate']:.1%}); contested {w['contested_agree']}/{w['contested_total']}")
    if f["supported"]:
        print(f"fold: splits={f['grapheme_splits']} concat_bad={f['concat_mismatch']} too_wide={f['line_too_wide']} "
              f"not_greedy={f['valid_but_not_greedy']} missing={f['missing']} of {f['total']}")
    if t["supported"]:
        print(f"truncate: splits={t['grapheme_splits']} not_prefix={t['not_a_prefix']} not_maximal={t['valid_but_not_maximal']} missing={t['missing']}")
    if tb["supported"]:
        print(f"tables: misaligned rows {tb['misaligned_rows']}/{tb['rows']} in {tb['tables_with_misalignment']}/{tb['tables']} tables")
    wg = report["widgets"]
    if wg["supported"]:
        print(f"panels and trees: rows wider than the terminal {wg['rows_wider_than_terminal']}/{wg['rows']}, "
              f"panel rows misaligned {wg['panel_rows_misaligned']}, outputs {wg['outputs']}, missing {wg['missing']}")
    print("unsupported:", [k for k, v in report.items() if not v["supported"]] or "none")


if __name__ == "__main__":
    main()
