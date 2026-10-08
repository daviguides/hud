"""Generate the Live, Layout and Columns corpus (cases/live_layout.jsonl, corpus 3).

Deterministic: same script, same file. A separate file and a separate seed range leave the corpus 1
cases and goldens as they were (see spec/live-layout-schema.md). Reuses the item generators of
gen_cases.py so the nodes inside a grid, a layout or a frame are the ones of the base corpus.
"""

import json
import random
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

from gen_cases import BOXES, COLOR_SYSTEMS, NAMES, STATUS, WIDTHS, panel_body, tree_node, words  # noqa: E402

OUT = HERE.parent / "cases" / "live_layout.jsonl"
CORPUS_VERSION = "3"
PER_FEATURE = 30
HEIGHTS = [8, 10, 12, 16, 24]


def base(feature, idx, rng, height=24):
    return {
        "id": f"{feature}-{idx:03d}",
        "feature": feature,
        "width": rng.choice(WIDTHS),
        "height": height,
        "color_system": COLOR_SYSTEMS[idx % 4],
        "corpus": CORPUS_VERSION,
    }


def small_table(rng, nrows=None):
    return {
        "t": "table", "title": None, "caption": None, "box": rng.choice(BOXES), "show_lines": False,
        "columns": [{"header": "Key", "justify": "left"}, {"header": "Value", "justify": "right"}],
        "rows": [[rng.choice(NAMES), str(rng.randint(1, 999))] for _ in range(nrows or rng.randint(2, 4))],
    }


def small_tree(rng):
    root = tree_node(rng, 0, rng.randint(1, 2))
    root["label"] = rng.choice(["src/", "[bold]repo[/bold]", "root"])
    return {"t": "tree", "guide_style": rng.choice(["", "dim", "green"]), "root": root}


def small_panel(rng):
    return {"t": "panel", "title": words(rng, 1), "subtitle": None, "box": rng.choice(BOXES), "expand": False,
            "padding": [0, 1], "border_style": rng.choice(["", "cyan", "red"]),
            "body": {"t": "text", "markup": words(rng, rng.randint(2, 5))}}


def item_text(rng):
    kind = rng.random()
    if kind < 0.45:
        return rng.choice(NAMES) + ("-" + words(rng, 1) if rng.random() < 0.5 else "")
    if kind < 0.7:
        return f"[{rng.choice(['green', 'red', 'yellow', 'cyan'])}]{rng.choice(STATUS)}[/] {words(rng, rng.randint(0, 2))}".strip()
    if kind < 0.85:
        return f"[bold]{words(rng, 1)}[/bold] {words(rng, rng.randint(1, 4))}"
    return words(rng, rng.randint(1, 6))


def item(rng, rich):
    if rich and rng.random() < 0.3:
        return rng.choice([small_panel, small_table, small_tree])(rng)
    return {"t": "text", "markup": item_text(rng)}


def columns_cases():
    rng = random.Random(301)
    cases = []
    paddings = [[0, 1], [0, 1], [0, 2], [1, 1], [0, 0], [0, 3], [1, 2], [0, 1, 0, 2], [0, 2, 0, 1], [1, 1, 0, 0]]
    for i in range(PER_FEATURE):
        c = base("columns", i, rng)
        rich = i % 5 == 4
        count = rng.randint(3, 14) if not rich else rng.randint(3, 8)
        node = {
            "t": "columns",
            "items": [item(rng, rich) for _ in range(count)],
            "padding": paddings[i % len(paddings)],
            "width": None,
            "expand": i % 3 == 1,
            "equal": i % 4 == 1,
            "column_first": i % 5 in (1, 3),
            "right_to_left": i % 6 == 5,
            "align": [None, None, "left", "center", "right"][i % 5],
            "title": words(rng, 2) if i % 7 == 0 else None,
        }
        if i % 8 == 3:
            node["width"] = rng.choice([8, 10, 14, 18])
        c["renderable"] = node
        cases.append(c)
    return cases


def layout_leaf(rng, depth):
    kind = rng.random()
    if kind < 0.55:
        return {"t": "text", "markup": words(rng, rng.randint(2, 18)) if rng.random() < 0.7
                else f"[bold]{words(rng, 1)}[/bold] " + words(rng, rng.randint(3, 10))}
    if kind < 0.75:
        return {"t": "panel", "title": words(rng, 1), "subtitle": None, "box": rng.choice(BOXES), "expand": True,
                "padding": [0, 1], "border_style": rng.choice(["", "blue", "green"]), "body": panel_body(rng, 1)}
    if kind < 0.9:
        return small_table(rng)
    return small_tree(rng)


def layout_node(rng, depth, max_depth, index_path):
    node = {"name": "n" + "".join(str(p) for p in index_path) if index_path else "root"}
    if depth < max_depth and rng.random() < 0.85:
        node["splitter"] = rng.choice(["row", "column"])
        count = rng.randint(2, 4 if depth == 0 else 3)
        node["children"] = []
        for position in range(count):
            child = layout_node(rng, depth + 1, max_depth, index_path + [position])
            if rng.random() < 0.35:
                child["size"] = rng.choice([3, 4, 5, 8, 12, 20])
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
        node["renderable"] = layout_leaf(rng, depth)
    return node


def layout_cases():
    rng = random.Random(302)
    cases = []
    for i in range(PER_FEATURE):
        c = base("layout", i, rng, height=HEIGHTS[i % len(HEIGHTS)])
        root = layout_node(rng, 0, rng.randint(1, 3), [])
        if "children" not in root:
            root = {"name": "root", "splitter": rng.choice(["row", "column"]),
                    "children": [{"name": "n0", "renderable": layout_leaf(rng, 1)},
                                 {"name": "n1", "renderable": layout_leaf(rng, 1)}]}
        c["renderable"] = {"t": "layout", "root": root}
        cases.append(c)
    return cases


def frame_text(rng, step, total):
    return {"t": "text", "markup": f"[bold]step {step + 1}[/bold] of {total}: {words(rng, rng.randint(2, 6))}"}


def frame_table(rng, rows):
    table = small_table(rng, rows)
    table["box"] = "simple"
    return table


def frame_panel(rng, lines):
    return {"t": "panel", "title": f"{lines} lines", "subtitle": None, "box": "rounded", "expand": True,
            "padding": [0, 1], "border_style": "", "body": {"t": "text", "markup": "\n".join(words(rng, 2) for _ in range(lines))}}


def live_cases():
    rng = random.Random(303)
    cases = []
    modes = ["ellipsis", "crop", "visible"]
    for i in range(PER_FEATURE):
        overflow = i % 5 >= 3
        height = rng.choice([6, 7, 8]) if overflow else rng.choice([10, 16, 24])
        c = base("live", i, rng, height=height)
        total = rng.randint(1, 6)
        kind = i % 4
        frames = []
        for step in range(total):
            if overflow:
                frames.append(frame_panel(rng, height + rng.randint(0, 4)) if kind % 2 == 0
                              else frame_table(rng, height + step))
            elif kind == 0:
                frames.append(frame_text(rng, step, total))
            elif kind == 1:
                frames.append(frame_table(rng, 1 + step))
            elif kind == 2:
                frames.append(frame_panel(rng, 1 + (step * 2) % 5))
            else:
                frames.append(small_panel(rng) if step % 2 == 0 else frame_text(rng, step, total))
        c["renderable"] = {
            "t": "live",
            "frames": frames,
            "transient": i % 3 == 2,
            "vertical_overflow": modes[i % 3] if overflow else "ellipsis",
            "terminal": i % 7 != 6,
        }
        cases.append(c)
    return cases


def main():
    cases = columns_cases() + layout_cases() + live_cases()
    OUT.parent.mkdir(parents=True, exist_ok=True)
    with OUT.open("w") as f:
        for c in cases:
            f.write(json.dumps(c, ensure_ascii=False, sort_keys=True) + "\n")
    print(f"{len(cases)} cases -> {OUT}")


if __name__ == "__main__":
    main()
