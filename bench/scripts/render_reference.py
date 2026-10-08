"""Reference renderer: declarative case -> ANSI bytes, using pinned Python Rich.

Used to generate golden fixtures and as the harness self-test (Rich must score
100% on its own corpus). Candidate adapters implement the same schema.
"""

import io
import json
import os
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

from rich import box as rbox
from rich.console import Console, Group
from rich.padding import Padding
from rich.panel import Panel
from rich.progress import BarColumn, MofNCompleteColumn, Progress, TaskProgressColumn, TextColumn
from rich.table import Table
from rich.text import Text
from rich.tree import Tree

BOXES = {
    "rounded": rbox.ROUNDED, "ascii": rbox.ASCII, "simple": rbox.SIMPLE, "heavy": rbox.HEAVY,
    "double": rbox.DOUBLE, "minimal": rbox.MINIMAL, "square": rbox.SQUARE,
    "heavy_head": rbox.HEAVY_HEAD,
}
COLOR_SYSTEMS = {"truecolor": "truecolor", "256": "256", "standard": "standard", "none": None}


def make_console(width, color_system, file=None):
    return Console(
        file=file or io.StringIO(),
        width=width,
        height=24,
        force_terminal=True,
        color_system=COLOR_SYSTEMS[color_system],
        legacy_windows=False,
        emoji=False,
        highlight=False,
        _environ={},
    )


def build(node):
    t = node["t"]
    if t == "text":
        if "markup" in node:
            return Text.from_markup(node["markup"])
        return Text(node["plain"], style=node.get("style", ""))
    if t == "table":
        table = Table(
            title=node.get("title"),
            caption=node.get("caption"),
            box=BOXES[node["box"]],
            show_lines=node.get("show_lines", False),
        )
        for col in node["columns"]:
            table.add_column(
                col["header"],
                justify=col.get("justify", "left"),
                style=col.get("style", ""),
                no_wrap=col.get("no_wrap", False),
            )
        for row in node["rows"]:
            table.add_row(*row)
        return table
    if t == "panel":
        pad = node.get("padding", [0, 1])
        return Panel(
            build(node["body"]),
            title=node.get("title"),
            subtitle=node.get("subtitle"),
            box=BOXES[node["box"]],
            expand=node.get("expand", True),
            padding=(pad[0], pad[1]),
            border_style=node.get("border_style", ""),
        )
    if t == "tree":
        def add(parent, child):
            sub = parent.add(child["label"])
            for c in child["children"]:
                add(sub, c)
        root = node["root"]
        tree = Tree(root["label"], guide_style=node.get("guide_style", ""))
        for c in root["children"]:
            add(tree, c)
        return tree
    if t == "progress":
        progress = Progress(
            TextColumn("{task.description}"),
            BarColumn(bar_width=node["bar_width"]),
            TaskProgressColumn(),
            MofNCompleteColumn(),
            auto_refresh=False,
            console=Console(file=io.StringIO(), _environ={}),
        )
        for task in node["tasks"]:
            progress.add_task(task["description"], total=task["total"], completed=task["completed"])
        return progress.get_renderable()
    if t == "error":
        lines = [Text(node["message"], style="bold")]
        causes = node.get("causes", [])
        if causes:
            lines.append(Text(""))
            lines.append(Text("Caused by:", style="dim"))
            for i, cause in enumerate(causes):
                lines.append(Text(f"    {i}: {cause}"))
        if node.get("hint"):
            lines.append(Text(""))
            lines.append(Text("hint: " + node["hint"], style="cyan"))
        return Panel(Group(*lines), title="Error", box=rbox.ROUNDED, border_style="red")
    raise ValueError(f"unknown renderable {t}")


def render_case(case):
    console = make_console(case["width"], case["color_system"])
    console.print(build(case["renderable"]))
    return console.file.getvalue().encode("utf-8")


def load_cases(path):
    with open(path) as f:
        return [json.loads(line) for line in f if line.strip()]


def render_case_isolated(case):
    """Render one case in a fresh interpreter.

    Rich caches a Style's ANSI codes on the Style object on first use whatever the color system, and
    Style.parse is lru_cached, so rendering cases one after another in one process makes a case's bytes
    depend on which color depth earlier cases used. A fresh process per case removes that order dependence.
    """
    out = subprocess.run([sys.executable, str(Path(__file__).resolve()), "--one"], input=json.dumps(case).encode(),
                         stdout=subprocess.PIPE, check=True)
    return out.stdout


def main():
    import argparse

    ap = argparse.ArgumentParser(description="Render a declarative corpus with Python Rich")
    ap.add_argument("cases", type=Path, nargs="?")
    ap.add_argument("outdir", type=Path, nargs="?")
    ap.add_argument("--one", action="store_true", help="render the case JSON on stdin to stdout (internal)")
    args = ap.parse_args()
    if args.one:
        sys.stdout.buffer.write(render_case(json.loads(sys.stdin.read())))
        return
    args.outdir.mkdir(parents=True, exist_ok=True)
    cases = load_cases(args.cases)

    def one(case):
        (args.outdir / f"{case['id']}.ansi").write_bytes(render_case_isolated(case))

    with ThreadPoolExecutor(max_workers=min(8, os.cpu_count() or 1)) as pool:
        list(pool.map(one, cases))
    print(f"{len(cases)} cases rendered into {args.outdir}, one fresh process per case")


if __name__ == "__main__":
    main()
