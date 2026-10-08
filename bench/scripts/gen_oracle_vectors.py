"""Differential test vectors: random styles and markup rendered by the pinned Rich.

The corpus (cases/correctness.jsonl) is the gate; these vectors are a development oracle that
reaches far past it: every color name, every palette index, thousands of RGB values through the
256-color and 16-color downgrade, and random markup and styled text with wrapping, tabs and
justification. The Rust test `crates/hud/tests/oracle.rs` reads them. Deterministic (seeded).

Rich caches a Style's escape codes on first use whatever the color system, so every markup case
renders in a fresh process, and the style vectors render each input in one fresh process with the
caches cleared between color systems (verified against fully fresh processes on a sample).
"""

import json
import random
import sys
from multiprocessing import Pool
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

ROOT = HERE.parents[1]
OUT = ROOT / "crates" / "hud" / "tests" / "fixtures"
SEED = 20261008

ATTRS = ["bold", "b", "dim", "d", "italic", "i", "underline", "u", "blink", "blink2", "reverse", "r",
         "conceal", "c", "strike", "s", "underline2", "uu", "frame", "encircle", "overline", "o"]
NAMED = ["black", "red", "green", "yellow", "blue", "magenta", "cyan", "white", "bright_black", "bright_red",
         "bright_green", "bright_yellow", "bright_blue", "bright_magenta", "bright_cyan", "bright_white",
         "grey50", "gray37", "navy_blue", "dark_orange", "deep_pink4", "default", "orange1", "purple"]
ASCII_WORDS = ["alpha", "beta", "gamma", "delta", "lexer", "parser", "worker", "x", "ok", "failed", "retry",
               "supercalifragilisticexpialidocious", "a", "bb", "ccc", "deploy", "12.5s", "[x]", "a-b", "end."]
UNICODE_WORDS = ["你好世界", "日本語", "한국어", "é", "é", "ｆｕｌｌ", "😀", "👍🏽", "🇧🇷", "naïve", "café", "😀😀😀"]
BAD_STYLES = ["nonsense", "bold nonsense", "color(300)", "#fff", "rgb(1,2)", "on", "not", "red on", "not red",
              "link", "bold,red", "x y z"]


def color_word(rng):
    kind = rng.random()
    if kind < 0.45:
        return rng.choice(NAMED)
    if kind < 0.6:
        return f"color({rng.randrange(256)})"
    if kind < 0.85:
        return "#%06x" % rng.randrange(1 << 24)
    return "rgb(%d,%d,%d)" % (rng.randrange(256), rng.randrange(256), rng.randrange(256))


def style_string(rng, allow_bad=True):
    if allow_bad and rng.random() < 0.08:
        return rng.choice(BAD_STYLES)
    words = []
    for _ in range(rng.randrange(1, 5)):
        r = rng.random()
        if r < 0.45:
            words.append(rng.choice(ATTRS))
        elif r < 0.55:
            words.append("not " + rng.choice(ATTRS))
        elif r < 0.8:
            words.append(color_word(rng))
        else:
            words.append("on " + color_word(rng))
    rng.shuffle(words) if rng.random() < 0.3 else None
    return " ".join(words)


def style_inputs(rng):
    from rich.color import ANSI_COLOR_NAMES

    inputs = []
    for name in sorted(ANSI_COLOR_NAMES):
        inputs += [name, "on " + name, name.upper(), f"bold {name} on {name}"]
    inputs += [f"color({n})" for n in range(256)]
    inputs += [f"on color({n})" for n in range(256)]
    edges = [0, 1, 46, 47, 48, 94, 95, 96, 115, 116, 135, 136, 154, 155, 175, 215, 216, 254, 255]
    for r in edges:
        for g in (0, 95, 135, 255):
            for b in (0, 47, 155, 255):
                inputs.append("#%02x%02x%02x" % (r, g, b))
    for v in range(0, 256, 3):
        inputs.append("#%02x%02x%02x" % (v, v, v))
        inputs.append("#%02x%02x%02x" % (v, v, min(255, v + 12)))
    inputs += ["#%06x" % rng.randrange(1 << 24) for _ in range(2500)]
    inputs += ["rgb(%d, %d,%d)" % (rng.randrange(256), rng.randrange(256), rng.randrange(256)) for _ in range(300)]
    inputs += [style_string(rng) for _ in range(900)]
    inputs += ATTRS + ["not " + a for a in ATTRS] + ["", "none", "  ", "BOLD", "Bold Red", "rgb(1,2,3,4)",
                                                       "rgb(256,0,0)", "color(256)", "#GGGGGG", "#12345", "#1234567"]
    inputs += BAD_STYLES
    seen, unique = set(), []
    for item in inputs:
        if item not in seen:
            seen.add(item)
            unique.append(item)
    return unique


def style_vector(item):
    from rich.color import ColorSystem
    from rich.style import Style

    systems = {"standard": ColorSystem.STANDARD, "256": ColorSystem.EIGHT_BIT, "truecolor": ColorSystem.TRUECOLOR}
    try:
        Style.parse(item)
    except Exception:
        return {"input": item, "ok": False}
    out = {"input": item, "ok": True, "ansi": {}}
    for name, system in systems.items():
        Style.parse.cache_clear()
        Style._add.cache_clear()
        out["ansi"][name] = Style.parse(item).render("X", color_system=system)
    Style.parse.cache_clear()
    out["canonical"] = str(Style.parse(item))
    return out


def style_fresh(item_system):
    item, name = item_system
    from rich.color import ColorSystem
    from rich.style import Style

    system = {"standard": ColorSystem.STANDARD, "256": ColorSystem.EIGHT_BIT, "truecolor": ColorSystem.TRUECOLOR}[name]
    return Style.parse(item).render("X", color_system=system)


def markup_text(rng, unicode_words):
    pool = ASCII_WORDS + (UNICODE_WORDS if unicode_words else [])
    out, open_tags = [], []
    for _ in range(rng.randrange(1, 14)):
        r = rng.random()
        if r < 0.45:
            out.append(rng.choice(pool))
        elif r < 0.6:
            out.append(rng.choice([" ", " ", "  ", "   ", "\t", "\n"]))
        elif r < 0.75:
            style = style_string(rng) if rng.random() < 0.8 else rng.choice(["BOLD", "Bold", "x", "link=https://x.org"])
            out.append(f"[{style}]")
            open_tags.append(style)
        elif r < 0.9:
            if open_tags and rng.random() < 0.85:
                style = open_tags.pop(rng.randrange(len(open_tags))) if rng.random() < 0.3 else open_tags.pop()
                out.append("[/]" if rng.random() < 0.5 else f"[/{style}]")
            else:
                out.append(rng.choice(["[/]", "[/bold]", "[/nope]"]))
        elif r < 0.95:
            out.append(rng.choice(["\\[", "\\\\", "\\[bold]", "\\\\[bold]", "[1]", "[]", "[ ]", "a\\[1]"]))
        else:
            out.append(rng.choice(pool) + " ")
    return "".join(out)


def make_markup_case(rng, index, unicode_words):
    return {
        "id": f"{'mu' if unicode_words else 'ma'}-{index:04d}",
        "kind": "markup",
        "markup": markup_text(rng, unicode_words),
        "width": rng.choice([5, 8, 10, 12, 20, 30, 40, 60, 80]),
        "color_system": rng.choice(["truecolor", "256", "standard", "none"]),
        "justify": rng.choice(["default"] * 6 + ["left", "center", "right", "full"]),
        "overflow": rng.choice(["fold"] * 6 + ["crop", "ellipsis", "ignore"]),
        "no_wrap": rng.random() < 0.1,
        "tab_size": rng.choice([8] * 4 + [2, 4, 5]),
        "end": rng.choice(["\n"] * 6 + ["", " |", "\n\n"]),
    }


def make_styled_case(rng, index, unicode_words):
    pool = ASCII_WORDS + (UNICODE_WORDS if unicode_words else [])
    plain = "".join(rng.choice(pool) + rng.choice([" ", "  ", "\t", ""]) for _ in range(rng.randrange(1, 9)))
    return {
        "id": f"{'su' if unicode_words else 'sa'}-{index:04d}",
        "kind": "styled",
        "plain": plain,
        "style": style_string(rng, allow_bad=False),
        "width": rng.choice([8, 12, 20, 40, 80]),
        "color_system": rng.choice(["truecolor", "256", "standard", "none"]),
        "justify": rng.choice(["default"] * 5 + ["left", "center", "right", "full"]),
        "overflow": "fold",
        "no_wrap": False,
        "tab_size": 8,
        "end": "\n",
    }


BOXES = ["rounded", "ascii", "simple", "heavy", "double", "minimal", "square", "heavy_head"]


def cell_markup(rng, unicode_words):
    pool = ASCII_WORDS + (UNICODE_WORDS if unicode_words else [])
    out = []
    for _ in range(rng.randrange(0, 6)):
        word = rng.choice(pool)
        r = rng.random()
        if r < 0.25:
            out.append(f"[{style_string(rng, allow_bad=False)}]{word}[/]")
        elif r < 0.3:
            out.append(rng.choice(["\\[x]", "\\[", "\\\\"]))
        else:
            out.append(word)
        out.append(rng.choice([" ", " ", " ", "  ", "\n", ""]))
    return "".join(out)


def make_table_case(rng, index, unicode_words):
    count = rng.randrange(1, 7)
    columns = []
    for c in range(count):
        column = {
            "header": cell_markup(rng, unicode_words) if rng.random() < 0.7 else f"H{c}",
            "justify": rng.choice(["left", "left", "center", "right"]),
            "style": style_string(rng, allow_bad=False) if rng.random() < 0.3 else "",
            "no_wrap": rng.random() < 0.15,
            "overflow": rng.choice(["ellipsis"] * 6 + ["fold", "crop", "ignore"]),
        }
        if rng.random() < 0.1:
            column["width"] = rng.randrange(1, 25)
        if rng.random() < 0.1:
            column["min_width"] = rng.randrange(1, 20)
        if rng.random() < 0.1:
            column["max_width"] = rng.randrange(3, 30)
        if rng.random() < 0.15:
            column["header_style"] = style_string(rng, allow_bad=False)
        columns.append(column)
    rows = []
    for _ in range(rng.randrange(0, 9)):
        cells = count if rng.random() < 0.9 else rng.randrange(0, count + 2)
        rows.append([cell_markup(rng, unicode_words) for _ in range(cells)])
    return {
        "id": f"{'tu' if unicode_words else 'ta'}-{index:04d}",
        "kind": "table",
        "box": rng.choice(BOXES),
        "show_lines": rng.random() < 0.3,
        "title": cell_markup(rng, unicode_words) if rng.random() < 0.4 else None,
        "caption": cell_markup(rng, unicode_words) if rng.random() < 0.3 else None,
        "header_style": style_string(rng, allow_bad=False) if rng.random() < 0.2 else None,
        "columns": columns,
        "rows": rows,
        "width": rng.choice([10, 16, 20, 30, 40, 60, 80, 100, 120]),
        "color_system": rng.choice(["truecolor", "256", "standard", "none"]),
    }


def render_table_case(case):
    from rich import box as rbox
    from rich.table import Table

    from render_reference import make_console

    console = make_console(case["width"], case["color_system"])
    try:
        kwargs = {}
        if case["header_style"] is not None:
            kwargs["header_style"] = case["header_style"]
        table = Table(title=case["title"], caption=case["caption"], box=getattr(rbox, case["box"].upper()),
                      show_lines=case["show_lines"], **kwargs)
        for col in case["columns"]:
            table.add_column(
                col["header"], justify=col["justify"], style=col["style"], no_wrap=col["no_wrap"],
                overflow=col["overflow"], width=col.get("width"), min_width=col.get("min_width"),
                max_width=col.get("max_width"), header_style=col.get("header_style"),
            )
        for row in case["rows"]:
            table.add_row(*row)
        console.print(table)
    except Exception as error:
        return {**case, "error": type(error).__name__}
    return {**case, "ansi": console.file.getvalue()}


def render_case(case):
    from rich.console import Group
    from rich.text import Text

    from render_reference import make_console

    if case["kind"] == "table":
        return render_table_case(case)
    console = make_console(case["width"], case["color_system"])
    try:
        if case["kind"] == "markup":
            text = Text.from_markup(case["markup"])
        else:
            text = Text(case["plain"], style=case["style"])
        # Console.print joins plain Text arguments into a new Text (dropping their own justify,
        # overflow and no_wrap) and wraps `justify=` in an Align of the whole block. A Text inside
        # a Group is rendered as a renderable of its own, which is what hud's Text means.
        text.justify = None if case["justify"] == "default" else case["justify"]
        text.overflow = case["overflow"]
        text.no_wrap = case["no_wrap"]
        text.tab_size = case["tab_size"]
        text.end = case["end"]
        console.print(Group(text))
    except Exception as error:
        return {**case, "error": type(error).__name__}
    return {**case, "ansi": console.file.getvalue()}


def panel_title(rng, unicode_words):
    return cell_markup(rng, unicode_words) if rng.random() < 0.55 else None


def make_node(rng, unicode_words, depth):
    kinds = ["text", "text", "text", "table", "tree"] + (["panel", "panel"] if depth < 2 else [])
    kind = rng.choice(kinds)
    if kind == "text":
        return {"t": "text", "markup": cell_markup(rng, unicode_words) or "x"}
    if kind == "table":
        case = make_table_case(rng, 0, unicode_words)
        return {"t": "table", "title": case["title"], "caption": case["caption"], "box": case["box"],
                "show_lines": case["show_lines"],
                "columns": [{k: c[k] for k in ("header", "justify", "style", "no_wrap")} for c in case["columns"]],
                "rows": case["rows"]}
    if kind == "tree":
        return make_tree_node(rng, unicode_words)
    return make_panel_node(rng, unicode_words, depth + 1)


def make_panel_node(rng, unicode_words, depth):
    padding = rng.choice([[0, 1]] * 3 + [[0, 0], [1, 2], [0, 2], [1, 1], [2, 3], [1, 0], [1], [2], [0, 1, 2, 3], [1, 0, 2, 4]])
    return {
        "t": "panel",
        "body": make_node(rng, unicode_words, depth),
        "box": rng.choice(BOXES),
        "expand": rng.random() < 0.5,
        "padding": padding,
        "title": panel_title(rng, unicode_words),
        "subtitle": panel_title(rng, unicode_words) if rng.random() < 0.5 else None,
        "title_align": rng.choice(["center"] * 4 + ["left", "right"]),
        "subtitle_align": rng.choice(["center"] * 4 + ["left", "right"]),
        "border_style": style_string(rng, allow_bad=False) if rng.random() < 0.6 else "",
    }


def tree_children(rng, unicode_words, depth):
    if depth >= 4:
        return []
    out = []
    for _ in range(rng.choice([0, 0, 1, 1, 2, 3])):
        child = {"label": cell_markup(rng, unicode_words) if rng.random() < 0.5 else rng.choice(ASCII_WORDS),
                 "children": tree_children(rng, unicode_words, depth + 1)}
        if rng.random() < 0.12:
            child["guide_style"] = style_string(rng, allow_bad=False)
        out.append(child)
    return out


def make_tree_node(rng, unicode_words):
    guide = rng.choice(["", "dim", "bold", "underline2", "uu", "bold #ff8800", "green", "dim red", "not bold",
                        style_string(rng, allow_bad=False)])
    return {"t": "tree", "guide_style": guide,
            "root": {"label": rng.choice(ASCII_WORDS), "children": tree_children(rng, unicode_words, 0)}}


def make_widget_case(rng, index, unicode_words, kind):
    node = make_panel_node(rng, unicode_words, 0) if kind == "panel" else make_tree_node(rng, unicode_words)
    return {
        "id": f"{'p' if kind == 'panel' else 'r'}{'u' if unicode_words else 'a'}-{index:04d}",
        "kind": kind,
        "node": node,
        "width": rng.choice([10, 16, 20, 30, 40, 60, 80, 100, 120]),
        "color_system": rng.choice(["truecolor", "256", "standard", "none"]),
    }


def oracle_build(node):
    from rich import box as rbox
    from rich.panel import Panel
    from rich.tree import Tree

    import render_reference

    t = node["t"]
    if t == "text":
        return node["markup"]
    if t == "table":
        return render_reference.build(node)
    if t == "panel":
        pad = node["padding"]
        return Panel(
            oracle_build(node["body"]), title=node["title"], subtitle=node["subtitle"],
            title_align=node["title_align"], subtitle_align=node["subtitle_align"],
            box=getattr(rbox, node["box"].upper()), expand=node["expand"],
            padding=pad[0] if len(pad) == 1 else tuple(pad), border_style=node["border_style"],
        )

    def add(parent, child):
        sub = parent.add(child["label"], guide_style=child.get("guide_style"))
        for c in child["children"]:
            add(sub, c)

    tree = Tree(node["root"]["label"], guide_style=node["guide_style"])
    for c in node["root"]["children"]:
        add(tree, c)
    return tree


def render_widget_case(case):
    from render_reference import make_console

    console = make_console(case["width"], case["color_system"])
    try:
        console.print(oracle_build(case["node"]))
    except Exception as error:
        return {**case, "error": type(error).__name__}
    return {**case, "ansi": console.file.getvalue()}


def main_widgets():
    """Panel and tree vectors only (v0.4); the rest of the fixtures are left as they are."""
    rng = random.Random(SEED + 4)
    OUT.mkdir(parents=True, exist_ok=True)
    jobs = {
        "panel_ascii.jsonl": [make_widget_case(rng, i, False, "panel") for i in range(1200)],
        "panel_unicode.jsonl": [make_widget_case(rng, i, True, "panel") for i in range(500)],
        "tree_ascii.jsonl": [make_widget_case(rng, i, False, "tree") for i in range(1200)],
        "tree_unicode.jsonl": [make_widget_case(rng, i, True, "tree") for i in range(400)],
    }
    with Pool(8, maxtasksperchild=1) as pool:
        for name, cases in jobs.items():
            write(name, pool.map(render_widget_case, cases, chunksize=1))


PROGRESS_TEMPLATES = ["{task.description}", "{task.description}", "[bold]{task.description}[/]",
                      "{task.completed}/{task.total} {task.description}", "{task.description} ({task.total})",
                      "[red]{task.completed}[/] done"]
PROGRESS_TOTALS = [1, 2, 3, 5, 10, 12, 20, 99, 100, 120, 250, 1000, 12500]
PROGRESS_GAPS = [0, 0.5, 1, 2, 5, 29, 31, 60, 3600, 90000]
FINISHED_TEXTS = [" ", "[green]ok[/]", "done"]


def make_progress_columns(rng):
    pool = [
        {"t": "text", "template": rng.choice(PROGRESS_TEMPLATES)},
        {"t": "bar", "bar_width": rng.choice([None, 4, 10, 20, 30, 40])},
        {"t": "percent"},
        {"t": "mofn", "separator": rng.choice(["/", " of ", "|"])},
        {"t": "elapsed"},
        {"t": "remaining", "compact": rng.random() < 0.5, "elapsed_when_finished": rng.random() < 0.3},
        {"t": "spinner", "speed": rng.choice([1.0, 2.0, 0.5]), "finished_text": rng.choice(FINISHED_TEXTS)},
    ]
    chosen = rng.sample(pool, rng.randrange(1, 6))
    if rng.random() < 0.7 and not any(c["t"] == "text" for c in chosen):
        chosen.insert(0, pool[0])
    return chosen


def make_progress_case(rng, index, unicode_words):
    words = ASCII_WORDS + (UNICODE_WORDS if unicode_words else [])
    start = rng.choice([0.0, 100.0, 1000.5])
    tasks, events = [], []
    for _ in range(rng.choice([0, 1, 1, 2, 3, 4, 6])):
        total = rng.choice(PROGRESS_TOTALS)
        description = " ".join(rng.choice(words) for _ in range(rng.randrange(1, 4)))
        tasks.append({"description": description, "total": total})
    for number, task in enumerate(tasks):
        events.append([start, "add", number])
    clock, state = start, [{"completed": 0, "total": task["total"]} for task in tasks]
    for _ in range(rng.randrange(0, 12) if tasks else 0):
        number = rng.randrange(len(tasks))
        clock += rng.choice(PROGRESS_GAPS)
        now = state[number]
        kind = rng.random()
        if kind < 0.6 and now["completed"] < now["total"]:
            amount = rng.randrange(1, now["total"] - now["completed"] + 1)
            now["completed"] += amount
            events.append([clock, "advance", number, amount])
        elif kind < 0.85:
            value = rng.randrange(0, now["total"] + 1)
            now["completed"] = value
            events.append([clock, "update", number, value])
        else:
            total = rng.choice([t for t in PROGRESS_TOTALS if t >= now["completed"]] or [now["total"]])
            now["total"] = total
            events.append([clock, "total", number, total])
    first = clock + rng.choice(PROGRESS_GAPS)
    return {
        "id": f"{'g' if unicode_words else 'h'}{'u' if unicode_words else 'a'}-{index:04d}",
        "kind": "progress",
        "columns": make_progress_columns(rng),
        "tasks": tasks,
        "events": events,
        "first": first,
        "now": first + rng.choice([0, 0.04, 0.085, 0.4, 1.0, 7.3]),
        "width": rng.choice([20, 30, 40, 60, 80, 100, 120]),
        "color_system": rng.choice(["truecolor", "256", "standard", "none"]),
    }


def render_progress_case(case):
    import io

    from rich.console import Console
    from rich.progress import (BarColumn, MofNCompleteColumn, Progress, SpinnerColumn, TaskProgressColumn,
                               TextColumn, TimeElapsedColumn, TimeRemainingColumn)

    from render_reference import make_console

    def column(spec):
        kind = spec["t"]
        if kind == "text":
            return TextColumn(spec["template"])
        if kind == "bar":
            return BarColumn(bar_width=spec["bar_width"])
        if kind == "percent":
            return TaskProgressColumn()
        if kind == "mofn":
            return MofNCompleteColumn(separator=spec["separator"])
        if kind == "elapsed":
            return TimeElapsedColumn()
        if kind == "remaining":
            return TimeRemainingColumn(compact=spec["compact"], elapsed_when_finished=spec["elapsed_when_finished"])
        return SpinnerColumn(speed=spec["speed"], finished_text=spec["finished_text"])

    now = [0.0]
    try:
        progress = Progress(*[column(c) for c in case["columns"]], get_time=lambda: now[0], auto_refresh=False,
                            console=Console(file=io.StringIO(), _environ={}))
        ids = []
        for at, kind, number, *rest in case["events"]:
            now[0] = at
            if kind == "add":
                task = case["tasks"][number]
                ids.append(progress.add_task(task["description"], total=task["total"]))
            elif kind == "advance":
                progress.advance(ids[number], rest[0])
            elif kind == "update":
                progress.update(ids[number], completed=rest[0])
            else:
                progress.update(ids[number], total=rest[0])
        now[0] = case["first"]
        make_console(case["width"], case["color_system"]).print(progress.get_renderable())
        now[0] = case["now"]
        console = make_console(case["width"], case["color_system"])
        console.print(progress.get_renderable())
    except Exception as error:
        return {**case, "error": type(error).__name__}
    return {**case, "ansi": console.file.getvalue()}


def main_progress():
    """Progress vectors only (v0.5): a fake clock, the same events replayed in Rich and in hud."""
    rng = random.Random(SEED + 5)
    OUT.mkdir(parents=True, exist_ok=True)
    jobs = {
        "progress_ascii.jsonl": [make_progress_case(rng, i, False) for i in range(1200)],
        "progress_unicode.jsonl": [make_progress_case(rng, i, True) for i in range(300)],
    }
    with Pool(8, maxtasksperchild=1) as pool:
        for name, cases in jobs.items():
            write(name, pool.map(render_progress_case, cases, chunksize=1))


ERROR_PIECES = ["[bold]x[/]", "[red", "a]b", "\\", "\t", "\u0007", "\n", "  ", "/etc/hud/config.toml",
                "No such file or directory (os error 2)", "os error 2", "connection reset by peer"]


def error_text(rng, unicode_words):
    pool = ASCII_WORDS + (UNICODE_WORDS if unicode_words else [])
    out = []
    for _ in range(rng.randrange(1, 14)):
        r = rng.random()
        out.append(rng.choice(ERROR_PIECES) if r < 0.15 else rng.choice(pool))
        out.append(rng.choice([" ", " ", " ", "  ", ""]))
    return "".join(out).strip(" ") or "x"


def make_error_case(rng, index, unicode_words):
    return {
        "id": f"{'eu' if unicode_words else 'ea'}-{index:04d}",
        "kind": "error",
        "node": {
            "t": "error",
            "message": error_text(rng, unicode_words),
            "causes": [error_text(rng, unicode_words) for _ in range(rng.choice([0, 0, 1, 1, 2, 3, 5]))],
            "hint": error_text(rng, unicode_words) if rng.random() < 0.5 else None,
        },
        "width": rng.choice([10, 16, 20, 30, 40, 60, 80, 100, 120]),
        "color_system": rng.choice(["truecolor", "256", "standard", "none"]),
    }


def render_error_case(case):
    from render_reference import build, make_console

    console = make_console(case["width"], case["color_system"])
    try:
        console.print(build(case["node"]))
    except Exception as error:
        return {**case, "error": type(error).__name__}
    return {**case, "ansi": console.file.getvalue()}


def main_errors():
    """Error report vectors only (v0.6); the rest of the fixtures are left as they are."""
    rng = random.Random(SEED + 6)
    OUT.mkdir(parents=True, exist_ok=True)
    jobs = {
        "error_ascii.jsonl": [make_error_case(rng, i, False) for i in range(1200)],
        "error_unicode.jsonl": [make_error_case(rng, i, True) for i in range(400)],
    }
    with Pool(8, maxtasksperchild=1) as pool:
        for name, cases in jobs.items():
            write(name, pool.map(render_error_case, cases, chunksize=1))


def write(name, rows):
    path = OUT / name
    with path.open("w") as f:
        for row in rows:
            f.write(json.dumps(row, ensure_ascii=False) + "\n")
    print(f"{name}: {len(rows)} rows", file=sys.stderr)


def main():
    rng = random.Random(SEED)
    OUT.mkdir(parents=True, exist_ok=True)
    inputs = style_inputs(rng)
    ascii_markup = [make_markup_case(rng, i, False) for i in range(1800)]
    unicode_markup = [make_markup_case(rng, i, True) for i in range(700)]
    ascii_styled = [make_styled_case(rng, i, False) for i in range(500)]
    unicode_styled = [make_styled_case(rng, i, True) for i in range(200)]
    ascii_tables = [make_table_case(rng, i, False) for i in range(1500)]
    unicode_tables = [make_table_case(rng, i, True) for i in range(600)]
    with Pool(8, maxtasksperchild=1) as pool:
        styles = pool.map(style_vector, inputs, chunksize=1)
        sample = rng.sample([s for s in styles if s["ok"]], 150)
        fresh_jobs = [(s["input"], name) for s in sample for name in ("standard", "256", "truecolor")]
        fresh = pool.map(style_fresh, fresh_jobs, chunksize=1)
        for (item, name), got in zip(fresh_jobs, fresh):
            want = next(s for s in sample if s["input"] == item)["ansi"][name]
            assert got == want, f"cache contamination for {item!r} {name}: {got!r} != {want!r}"
        write("style.jsonl", styles)
        write("markup_ascii.jsonl", pool.map(render_case, ascii_markup + ascii_styled, chunksize=1))
        write("markup_unicode.jsonl", pool.map(render_case, unicode_markup + unicode_styled, chunksize=1))
        write("table_ascii.jsonl", pool.map(render_case, ascii_tables, chunksize=1))
        write("table_unicode.jsonl", pool.map(render_case, unicode_tables, chunksize=1))
    import importlib.metadata as md

    (OUT / "manifest.json").write_text(json.dumps({
        "rich": md.version("rich"), "seed": SEED, "generator": "bench/scripts/gen_oracle_vectors.py",
        "note": "Differential oracle vectors, not the gate. Regenerate with bench/.venv/bin/python.",
    }, indent=1) + "\n")


if __name__ == "__main__":
    {"widgets": main_widgets, "progress": main_progress, "errors": main_errors}.get(sys.argv[1] if sys.argv[1:] else "", main)()
