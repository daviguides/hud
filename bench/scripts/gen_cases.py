"""Generate the declarative correctness corpus (cases/correctness.jsonl).

Deterministic: same script, same file. Every case is a language-neutral
description (see spec/case-schema.md) that drives Python Rich (reference) and
every candidate adapter.
"""

import json
import random
from pathlib import Path

OUT = Path(__file__).resolve().parent.parent / "cases" / "correctness.jsonl"
CORPUS_VERSION = "1"
PER_FEATURE = 30
COLOR_SYSTEMS = ["truecolor", "256", "standard", "none"]
WIDTHS = [40, 60, 80, 100, 120]
BOXES = ["rounded", "ascii", "simple", "heavy", "double", "minimal", "square", "heavy_head"]

WORDS = (
    "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron pi rho "
    "sigma tau upsilon phi chi psi omega build deploy cache index parser lexer render buffer "
    "stream socket thread worker queue token route layer shell"
).split()
NAMES = ["clap", "serde", "tokio", "hyper", "rayon", "regex", "anyhow", "tracing", "bytes", "syn"]
STATUS = ["ok", "warn", "fail", "skip", "pending"]
COLORS = [
    "red", "green", "blue", "yellow", "magenta", "cyan", "bright_red", "bright_blue",
    "bright_green", "color(208)", "color(17)", "color(240)", "#ff8800", "#00ffcc", "#808080",
    "rgb(12,200,99)", "grey50", "default",
]
ATTRS = ["bold", "italic", "underline", "strike", "dim", "reverse", "bold italic", "underline dim",
         "bold underline", "italic strike"]


def words(rng, n):
    return " ".join(rng.choice(WORDS) for _ in range(n))


def base(feature, idx, rng):
    return {
        "id": f"{feature}-{idx:03d}",
        "feature": feature,
        "width": rng.choice(WIDTHS),
        "color_system": COLOR_SYSTEMS[idx % 4],
    }


def style_cases():
    rng = random.Random(101)
    cases = []
    for i in range(PER_FEATURE):
        c = base("style", i, rng)
        parts = []
        if i % 3 != 2:
            parts.append(rng.choice(ATTRS))
        if i % 5 != 4:
            parts.append(rng.choice(COLORS))
        if i % 4 == 1:
            parts.append("on " + rng.choice(["red", "#223344", "color(240)", "blue", "rgb(250,240,10)"]))
        style = " ".join(parts) or "bold"
        c["renderable"] = {"t": "text", "plain": words(rng, rng.randint(2, 9)), "style": style}
        cases.append(c)
    return cases


MARKUP = [
    "[bold]bold[/bold] and plain",
    "[b]b[/b] [i]i[/i] [u]u[/u] [s]s[/s]",
    "[red]red[/red] [green]green[/green] [blue]blue[/blue]",
    "[bold red]bold red[/] then plain",
    "[bold]outer [italic]inner[/italic] outer again[/bold]",
    "[bold][red][underline]triple[/underline][/red][/bold]",
    "[#ff8800]hex[/#ff8800] [color(208)]idx[/color(208)] [rgb(12,200,99)]rgb[/]",
    "[on blue]bg only[/on blue]",
    "[white on red]white on red[/]",
    "[bold yellow on #223344]combined[/]",
    "\\[not a tag] stays literal",
    "[bold]a\\[1] b[/bold]",
    "[dim]dim[/dim] [reverse]reverse[/reverse]",
    "plain text with no tags at all",
    "[bold]unclosed run to end of string",
    "[red]r[/red][green]g[/green][blue]b[/blue] adjacent",
    "[bold]x[/] [bold]y[/] [bold]z[/]",
    "[italic cyan]italic cyan[/italic cyan] done",
    "[strike]struck[/strike] [underline]under[/underline]",
    "[bright_magenta]bright[/] [grey50]grey[/] [default]default[/]",
    "[bold]B[italic]BI[underline]BIU[/underline]BI[/italic]B[/bold]",
    "[red]long red run that is long enough to wrap onto a second line when the width is narrow[/red]",
    "head [bold]mid[/bold] tail [bold red]end[/]",
    "[b][i][u][s]all four[/s][/u][/i][/b]",
    "[color(17) on color(240)]palette on palette[/]",
    "line one\n[bold]line two[/bold]\nline three",
    "[reverse bold]rev bold[/]",
    "tab\tseparated [green]values[/green]\there",
    "[blue]a[/blue] [bold blue]b[/bold blue] [italic blue]c[/italic blue]",
    "[red]nested [green]green in red[/green] back to red[/red]",
]


def markup_cases():
    rng = random.Random(102)
    cases = []
    for i in range(PER_FEATURE):
        c = base("markup", i, rng)
        c["renderable"] = {"t": "text", "markup": MARKUP[i]}
        cases.append(c)
    return cases


def table_cases():
    rng = random.Random(103)
    cases = []
    for i in range(PER_FEATURE):
        c = base("table", i, rng)
        ncols = rng.randint(2, 5)
        nrows = rng.randint(1, 8)
        justs = [rng.choice(["left", "center", "right"]) for _ in range(ncols)]
        headers = [words(rng, 1).capitalize() for _ in range(ncols)]
        cols = []
        for j in range(ncols):
            col = {"header": headers[j], "justify": justs[j]}
            if rng.random() < 0.3:
                col["style"] = rng.choice(["cyan", "green", "bold", "dim", "#ff8800"])
            if rng.random() < 0.15:
                col["no_wrap"] = True
            cols.append(col)
        rows = []
        for _ in range(nrows):
            row = []
            for j in range(ncols):
                kind = rng.random()
                if kind < 0.25:
                    row.append(str(rng.randint(0, 99999)))
                elif kind < 0.45:
                    row.append(f"[{rng.choice(['green','red','yellow'])}]{rng.choice(STATUS)}[/]")
                elif kind < 0.55:
                    row.append(words(rng, rng.randint(6, 14)))
                else:
                    row.append(rng.choice(NAMES) + " " + words(rng, rng.randint(0, 2)))
            rows.append(row)
        c["renderable"] = {
            "t": "table",
            "title": words(rng, 2) if i % 3 == 0 else None,
            "caption": words(rng, 2) if i % 7 == 0 else None,
            "box": BOXES[i % len(BOXES)],
            "show_lines": i % 5 == 0,
            "columns": cols,
            "rows": rows,
        }
        cases.append(c)
    return cases


def panel_body(rng, depth=0):
    kind = rng.random()
    if depth == 0 and kind < 0.15:
        return {"t": "panel", "title": words(rng, 1), "box": rng.choice(BOXES), "expand": False,
                "padding": [0, 1], "body": {"t": "text", "markup": words(rng, 4)}}
    if kind < 0.3:
        return {"t": "table", "title": None, "caption": None, "box": "simple", "show_lines": False,
                "columns": [{"header": "Key", "justify": "left"}, {"header": "Value", "justify": "right"}],
                "rows": [[rng.choice(NAMES), str(rng.randint(1, 999))] for _ in range(rng.randint(2, 4))]}
    n = rng.randint(3, 40)
    txt = words(rng, n)
    if rng.random() < 0.4:
        txt = f"[bold]{txt.split()[0]}[/bold] " + " ".join(txt.split()[1:])
    return {"t": "text", "markup": txt}


def panel_cases():
    rng = random.Random(104)
    cases = []
    for i in range(PER_FEATURE):
        c = base("panel", i, rng)
        c["renderable"] = {
            "t": "panel",
            "title": words(rng, rng.randint(1, 3)) if i % 4 != 3 else None,
            "subtitle": words(rng, 2) if i % 6 == 0 else None,
            "box": BOXES[i % len(BOXES)],
            "expand": i % 3 != 2,
            "padding": [rng.randint(0, 1), rng.randint(0, 2)],
            "border_style": rng.choice(["", "red", "cyan", "#ff8800", "bold green"]),
            "body": panel_body(rng),
        }
        cases.append(c)
    return cases


def tree_node(rng, depth, max_depth):
    node = {"label": rng.choice(NAMES) + ("/" if depth < max_depth else ".rs")}
    if rng.random() < 0.3:
        node["label"] = f"[{rng.choice(['bold','green','cyan'])}]{node['label']}[/]"
    node["children"] = []
    if depth < max_depth:
        for _ in range(rng.randint(1, 3)):
            node["children"].append(tree_node(rng, depth + 1, max_depth))
    return node


def tree_cases():
    rng = random.Random(105)
    cases = []
    for i in range(PER_FEATURE):
        c = base("tree", i, rng)
        root = tree_node(rng, 0, rng.randint(1, 3))
        root["label"] = rng.choice(["project/", "[bold]repo[/bold]", "root", "[cyan]src/[/cyan]"])
        c["renderable"] = {"t": "tree", "guide_style": rng.choice(["", "dim", "green", "bold #ff8800"]),
                           "root": root}
        cases.append(c)
    return cases


def progress_cases():
    rng = random.Random(106)
    cases = []
    for i in range(PER_FEATURE):
        c = base("progress", i, rng)
        tasks = []
        for _ in range(rng.randint(1, 5)):
            total = rng.choice([10, 20, 100, 250, 1000])
            completed = rng.choice([0, total // 4, total // 2, total * 3 // 4, total - 1, total])
            tasks.append({"description": words(rng, rng.randint(1, 2)), "total": total, "completed": completed})
        c["renderable"] = {"t": "progress", "bar_width": rng.choice([10, 20, 30, 40]), "tasks": tasks}
        cases.append(c)
    return cases


MESSAGES = [
    "could not read config",
    "failed to connect to the registry at https://crates.io after three retries",
    "invalid value for field `edition`",
    "permission denied",
    "build script exited with a non-zero status while compiling a very long dependency name",
]
CAUSES = [
    "No such file or directory (os error 2)",
    "connection reset by peer",
    "expected one of `2015`, `2018`, `2021`, `2024`",
    "the lock file is held by another process, retry in a few seconds",
    "stream did not contain valid UTF-8",
    "timed out after 30s waiting for the response headers from the upstream server",
]


def error_cases():
    rng = random.Random(107)
    cases = []
    for i in range(PER_FEATURE):
        c = base("error", i, rng)
        c["renderable"] = {
            "t": "error",
            "message": rng.choice(MESSAGES),
            "causes": [rng.choice(CAUSES) for _ in range(i % 5)],
            "hint": "run again with --verbose for details" if i % 4 == 0 else None,
        }
        cases.append(c)
    return cases


UNICODE_CELLS = [
    "漢字日本語", "한국어", "ひらがな", "😀🚀", "👨‍👩‍👧‍👦", "🇧🇷", "e\u0301le\u0301ve", "कक्षा", "ＡＢＣ", "ábç", "α→β", "🏳️‍🌈 pride",
    "mix 日本 and ascii", "👍🏽", "①②③", "1️⃣2️⃣", "naïve café", "Zürich 東京 São Paulo",
]


def table_unicode_cases():
    rng = random.Random(108)
    cases = []
    for i in range(12):
        ncols = rng.randint(2, 4)
        nrows = rng.randint(2, 6)
        rows = [[rng.choice(UNICODE_CELLS) for _ in range(ncols)] for _ in range(nrows)]
        cases.append({
            "id": f"tw-{i:03d}",
            "feature": "table",
            "width": rng.choice([60, 80, 100]),
            "color_system": ["none", "truecolor"][i % 2],
            "corpus": CORPUS_VERSION,
            "renderable": {
                "t": "table", "title": None, "caption": None, "box": BOXES[i % len(BOXES)],
                "show_lines": i % 4 == 0,
                "columns": [{"header": f"H{j}", "justify": rng.choice(["left", "center", "right"])} for j in range(ncols)],
                "rows": rows,
            },
        })
    return cases


def main():
    cases = []
    for fn in (style_cases, markup_cases, table_cases, panel_cases, tree_cases, progress_cases, error_cases):
        cases.extend(fn())
    OUT.parent.mkdir(parents=True, exist_ok=True)
    with OUT.open("w") as f:
        for c in cases:
            c["corpus"] = CORPUS_VERSION
            f.write(json.dumps(c, ensure_ascii=False, sort_keys=True) + "\n")
    print(f"{len(cases)} cases -> {OUT}")
    tw = table_unicode_cases()
    path = OUT.with_name("table_unicode.jsonl")
    with path.open("w") as f:
        for c in tw:
            f.write(json.dumps(c, ensure_ascii=False, sort_keys=True) + "\n")
    print(f"{len(tw)} unicode table cases -> {path}")


if __name__ == "__main__":
    main()
