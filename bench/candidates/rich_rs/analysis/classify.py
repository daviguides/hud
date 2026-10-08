"""Failure attribution for the rich-rs correctness run (bench-design.md, pilot validity rule 5).

Usage: uv run python candidates/rich_rs/analysis/classify.py <cases.jsonl> <golden_dir> <cand_dir>

Writes <cand_dir>/classifications.json (read by scripts/compare.py) and <cand_dir>/classification_summary.json.
Rules, in order, applied only to cases whose bytes differ from the golden:

1. reference_quirk: the candidate bytes equal what Python Rich 15.0.0 prints for the case in a *fresh process*.
   The committed golden differs because Rich caches the ANSI codes of a parsed Style across consoles of
   different color depth, so the first depth that touched a style string in the generating process wins.
2. documented_deviation: a table with a no_wrap column whose visible text differs; rich_rs::Column documents
   "justify, vertical, overflow and no_wrap ... are not yet fully implemented" (table.rs, struct Column).
3. candidate_bug, tagged: sgr_spelling_only (same screen cells, different bytes), attrs_on_blank_cells_only,
   attrs_differ_text_same (visible style differs), text_differs.
"""

import collections
import json
import re
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from screen_equiv import snapshot  # noqa: E402

BENCH = Path(__file__).resolve().parents[3]
CHILD = r'''
import json, sys
sys.path.insert(0, "scripts")
from render_reference import render_case
sys.stdout.buffer.write(render_case(json.loads(sys.argv[1])))
'''


def fresh_render(case):
    return subprocess.run([sys.executable, "-c", CHILD, json.dumps(case)], capture_output=True, cwd=BENCH).stdout


def drop_blank_attrs(snap):
    """Ignore bold/italic/dim on blank cells with default colors: not visible to a person."""
    blank = " "
    return [[(c[0], c[1], c[2], False, False, c[5], c[6], c[7]) if (c[0] == blank and c[1] == "default" and c[2] == "default" and not (c[5] or c[6] or c[7])) else c
             for c in row] for row in snap]


def visible_text(snap):
    return [b"".join(c[0].encode() for c in row).rstrip() for row in snap]


def main():
    cases_path, golden, cand = (Path(a) for a in sys.argv[1:4])
    classes, tags = {}, collections.Counter()
    for line in cases_path.read_text().splitlines():
        if not line.strip():
            continue
        c = json.loads(line)
        g = (golden / f"{c['id']}.ansi").read_bytes()
        o = (cand / f"{c['id']}.ansi").read_bytes()
        if g == o:
            continue
        if o == fresh_render(c):
            classes[c["id"]] = {"class": "reference_quirk", "tag": "rich_style_cache",
                                "reason": "candidate equals a fresh-process Rich render; golden contaminated by Rich's per-Style ANSI cache"}
        else:
            sg, so = snapshot(g, c["width"]), snapshot(o, c["width"])
            node = c["renderable"]
            no_wrap = node.get("t") == "table" and any(col.get("no_wrap") for col in node["columns"])
            if sg == so:
                classes[c["id"]] = {"class": "candidate_bug", "tag": "sgr_spelling_only",
                                    "reason": "same screen cells, different SGR bytes (per-attribute resets instead of ESC[0m, or span splitting)"}
            elif no_wrap and visible_text(sg) != visible_text(so):
                classes[c["id"]] = {"class": "documented_deviation", "tag": "no_wrap_overflow",
                                    "reason": "Column docs: no_wrap/overflow not yet fully implemented"}
            elif drop_blank_attrs(sg) == drop_blank_attrs(so):
                classes[c["id"]] = {"class": "candidate_bug", "tag": "attrs_on_blank_cells_only",
                                    "reason": "identical on screen except bold/italic on blank padding cells (invisible)"}
            elif visible_text(sg) == visible_text(so):
                classes[c["id"]] = {"class": "candidate_bug", "tag": "attrs_differ_text_same", "reason": "same text, different visible style attributes"}
            else:
                classes[c["id"]] = {"class": "candidate_bug", "tag": "text_differs", "reason": "different visible text or layout"}
        tags[(c["feature"], classes[c["id"]]["class"], classes[c["id"]]["tag"])] += 1
    (cand / "classifications.json").write_text(json.dumps(classes, indent=1) + "\n")
    summary = [{"feature": f, "class": k, "tag": t, "n": n} for (f, k, t), n in sorted(tags.items())]
    (cand / "classification_summary.json").write_text(json.dumps(summary, indent=1) + "\n")
    print(collections.Counter(v["class"] for v in classes.values()), collections.Counter(v["tag"] for v in classes.values()))


if __name__ == "__main__":
    main()
