"""Differential test vectors for Columns, Layout, Live and Progress on Live: random cases rendered by
the pinned Rich 15.0.0, one fresh process per case.

The corpus (cases/live_layout.jsonl) is the gate; these vectors are a development oracle that reaches
past it. The Rust test `crates/hud/tests/oracle_live_layout.rs` reads them. Deterministic (seeded) and
separate from gen_oracle_vectors.py, so no existing fixture changes.

  gen_oracle_vectors_live_layout.py [columns|layout|live|progress|all] [--count N] [--unicode N]
"""

import argparse
import json
import random
import sys
from multiprocessing import Pool
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

from gen_cases import BOXES, COLOR_SYSTEMS, NAMES, STATUS, WIDTHS, panel_body, tree_node, words  # noqa: E402
from render_live_layout_reference import render_case_isolated  # noqa: E402

ROOT = HERE.parents[1]
OUT = ROOT / "crates" / "hud" / "tests" / "fixtures"
SEED = 20261008 + 6
UNICODE_WORDS = ["你好世界", "日本語", "한국어", "é", "é", "ｆｕｌｌ", "😀", "👍🏽", "🇧🇷", "naïve", "café", "😀😀😀"]


def text_markup(rng, unicode_):
    kind = rng.random()
    if kind < 0.4:
        out = rng.choice(NAMES) + ("-" + words(rng, 1) if rng.random() < 0.4 else "")
    elif kind < 0.65:
        out = f"[{rng.choice(['green', 'red', 'yellow', 'cyan', 'bold', 'italic', 'dim'])}]{rng.choice(STATUS)}[/] {words(rng, rng.randint(0, 3))}".strip()
    elif kind < 0.8:
        out = f"[bold]{words(rng, 1)}[/bold] {words(rng, rng.randint(1, 5))}"
    else:
        out = words(rng, rng.randint(1, 9))
    if unicode_ and rng.random() < 0.7:
        parts = out.split(" ")
        parts.insert(rng.randrange(len(parts) + 1), rng.choice(UNICODE_WORDS))
        out = " ".join(parts)
    return out


def table_node(rng, unicode_):
    ncols = rng.randint(1, 3)
    return {
        "t": "table", "title": None, "caption": None, "box": rng.choice(BOXES), "show_lines": rng.random() < 0.2,
        "columns": [{"header": text_markup(rng, False).split(" ")[0] or "k", "justify": rng.choice(["left", "right", "center"])}
                    for _ in range(ncols)],
        "rows": [[text_markup(rng, unicode_) for _ in range(ncols)] for _ in range(rng.randint(1, 4))],
    }


def panel_node(rng, unicode_, expand=False):
    return {"t": "panel", "title": words(rng, 1) if rng.random() < 0.6 else None, "subtitle": None, "box": rng.choice(BOXES),
            "expand": expand, "padding": [rng.randint(0, 1), rng.randint(0, 2)],
            "border_style": rng.choice(["", "cyan", "red", "bold green"]),
            "body": {"t": "text", "markup": text_markup(rng, unicode_)}}


def tree_item(rng, unicode_):
    root = tree_node(rng, 0, rng.randint(1, 2))
    root["label"] = rng.choice(["src/", "[bold]repo[/bold]", "root"])
    if unicode_:
        root["label"] += " " + rng.choice(UNICODE_WORDS)
    return {"t": "tree", "guide_style": rng.choice(["", "dim", "green"]), "root": root}


def item(rng, unicode_, rich):
    if rich and rng.random() < 0.35:
        return rng.choice([lambda: panel_node(rng, unicode_), lambda: table_node(rng, unicode_), lambda: tree_item(rng, unicode_)])()
    return {"t": "text", "markup": text_markup(rng, unicode_)}


def base(feature, index, rng, height=24):
    return {"id": f"{feature}-v{index:04d}", "feature": feature, "width": rng.choice(WIDTHS + [20, 30]),
            "height": height, "color_system": rng.choice(COLOR_SYSTEMS), "corpus": "3"}


PADDINGS = [[0, 1], [0, 1], [0, 2], [1, 1], [0, 0], [0, 3], [1, 2], [0, 1, 0, 2], [0, 2, 0, 1], [1, 1, 0, 0], [2, 2], [1, 0]]


def columns_case(index, rng, unicode_):
    rich = rng.random() < 0.3
    fixed = rng.random() < 0.2
    case = base("columns", index, rng)
    padding = rng.choice(PADDINGS)
    width_padding = max(padding[-1], padding[1]) if len(padding) == 4 else padding[1]
    case["renderable"] = {
        "t": "columns", "items": [item(rng, unicode_, rich and not fixed) for _ in range(rng.randint(1, 18))],
        "padding": padding, "width": min(rng.choice([6, 8, 10, 14, 18, 24]), case["width"] - width_padding) if fixed else None,
        "expand": rng.random() < 0.3, "equal": rng.random() < 0.3, "column_first": rng.random() < 0.4,
        "right_to_left": rng.random() < 0.25, "align": rng.choice([None, None, "left", "center", "right"]),
        "title": words(rng, 2) if rng.random() < 0.25 else None,
    }
    return case


def layout_node(rng, unicode_, depth, max_depth, path):
    node = {"name": "n" + "".join(map(str, path))}
    if depth < max_depth and rng.random() < 0.85:
        node["splitter"] = rng.choice(["row", "column"])
        node["children"] = []
        for position in range(rng.randint(2, 4 if depth == 0 else 3)):
            child = layout_node(rng, unicode_, depth + 1, max_depth, path + [position])
            if rng.random() < 0.3:
                child["size"] = rng.choice([4, 5, 8, 12, 20])
            if rng.random() < 0.4:
                child["ratio"] = rng.choice([1, 2, 3])
            if rng.random() < 0.15:
                child["minimum_size"] = rng.choice([3, 5, 9])
            if rng.random() < 0.12:
                child["visible"] = False
            node["children"].append(child)
        if all(child.get("visible") is False for child in node["children"]):
            node["children"][0].pop("visible")
    else:
        kind = rng.random()
        if kind < 0.55:
            leaf = {"t": "text", "markup": " ".join(text_markup(rng, unicode_) for _ in range(rng.randint(1, 4)))}
        elif kind < 0.75:
            leaf = panel_node(rng, unicode_, expand=True)
        elif kind < 0.9:
            leaf = table_node(rng, unicode_)
        else:
            leaf = tree_item(rng, unicode_)
        node["renderable"] = leaf
    return node


def layout_case(index, rng, unicode_):
    case = base("layout", index, rng, height=rng.choice([8, 10, 12, 16, 24]))
    root = layout_node(rng, unicode_, 0, rng.randint(1, 3), [])
    if "children" not in root:
        root = {"name": "root", "splitter": "row", "children": [
            {"name": "a", "renderable": {"t": "text", "markup": text_markup(rng, unicode_)}},
            {"name": "b", "renderable": {"t": "text", "markup": text_markup(rng, unicode_)}}]}
    case["renderable"] = {"t": "layout", "root": root}
    return case


def frame(rng, unicode_, height, overflow):
    if overflow:
        lines = height + rng.randint(0, 3)
        return {"t": "panel", "title": f"{lines} lines", "subtitle": None, "box": "rounded", "expand": True, "padding": [0, 1],
                "border_style": "", "body": {"t": "text", "markup": "\n".join(text_markup(rng, unicode_) for _ in range(lines))}}
    return rng.choice([
        lambda: {"t": "text", "markup": text_markup(rng, unicode_)},
        lambda: {"t": "text", "markup": "\n".join(text_markup(rng, unicode_) for _ in range(rng.randint(1, 5)))},
        lambda: table_node(rng, unicode_),
        lambda: panel_node(rng, unicode_, expand=rng.random() < 0.5),
        lambda: tree_item(rng, unicode_),
    ])()


def live_case(index, rng, unicode_):
    overflow = rng.random() < 0.3
    height = rng.choice([4, 6, 8]) if overflow else rng.choice([10, 16, 24])
    case = base("live", index, rng, height=height)
    case["renderable"] = {
        "t": "live", "frames": [frame(rng, unicode_, height, overflow) for _ in range(rng.randint(1, 6))],
        "transient": rng.random() < 0.4, "vertical_overflow": rng.choice(["ellipsis", "crop", "visible"]),
        "terminal": rng.random() < 0.8,
    }
    return case


def progress_case(index, rng, unicode_):
    case = base("progress_live", index, rng)
    tasks = rng.randint(1, 4)
    events, totals = [], []
    for t in range(tasks):
        total = rng.choice([3, 4, 8, 10, 20, 100])
        totals.append(total)
        events.append({"op": "add", "description": text_markup(rng, unicode_).split("[")[0].strip() or "task", "total": total})
        for _ in range(rng.randint(0, 3)):
            if rng.random() < 0.5:
                events.append({"op": "advance", "task": t, "amount": rng.randint(1, max(1, total // 2))})
            else:
                events.append({"op": "update", "task": t, "completed": rng.randint(0, total)})
            if rng.random() < 0.7:
                events.append({"op": "refresh"})
    case["renderable"] = {"t": "progress_live", "bar_width": rng.choice([8, 10, 20, 30]), "events": events,
                          "transient": rng.random() < 0.35, "terminal": rng.random() < 0.8}
    return case


MAKERS = {"columns": columns_case, "layout": layout_case, "live": live_case, "progress": progress_case}


def render(case):
    return {"case": case, "out": render_case_isolated(case).decode("utf-8")}


def write(feature, unicode_, count):
    rng = random.Random(SEED + (1000 if unicode_ else 0) + sum(map(ord, feature)))
    cases = [MAKERS[feature](i, rng, unicode_) for i in range(count)]
    with Pool(4) as pool:
        rows = pool.map(render, cases, chunksize=8)
    stem = "progress_live" if feature == "progress" else feature
    name = f"{stem}_{'unicode' if unicode_ else 'ascii'}.jsonl"
    with (OUT / name).open("w") as f:
        for row in rows:
            f.write(json.dumps(row, ensure_ascii=False, sort_keys=True) + "\n")
    print(f"{len(rows)} vectors -> {name}")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("mode", nargs="?", default="all", choices=[*MAKERS, "all"])
    ap.add_argument("--count", type=int, default=1100)
    ap.add_argument("--unicode", type=int, default=300)
    args = ap.parse_args()
    features = list(MAKERS) if args.mode == "all" else [args.mode]
    for feature in features:
        write(feature, False, args.count)
        write(feature, True, args.unicode)


if __name__ == "__main__":
    main()
