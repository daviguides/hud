"""Reference renderer for corpus 3 (Columns, Layout, Live): case -> ANSI bytes with pinned Python Rich.

Extends render_reference.py without editing it: the nodes of the base schema go through its `build`,
the new ones are built here. One fresh process per case (Rich caches a Style's escape codes on first
use whatever the color system, so a shared process would make bytes depend on the order of cases).
"""

import io
import json
import os
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

from rich.columns import Columns
from rich.console import Console
from rich.layout import Layout
from rich.live import Live

sys.path.insert(0, str(Path(__file__).resolve().parent))
from render_reference import COLOR_SYSTEMS, build as build_base  # noqa: E402


def make_console(case, file=None, terminal=True):
    return Console(
        file=file or io.StringIO(),
        width=case["width"],
        height=case.get("height", 24),
        force_terminal=terminal,
        color_system=COLOR_SYSTEMS[case["color_system"]],
        legacy_windows=False,
        emoji=False,
        highlight=False,
        _environ={},
    )


def build(node):
    t = node["t"]
    if t == "columns":
        padding = tuple(node.get("padding", [0, 1]))
        return Columns(
            [build(item) for item in node["items"]],
            padding=padding if len(padding) != 1 else padding[0],
            width=node.get("width"),
            expand=node.get("expand", False),
            equal=node.get("equal", False),
            column_first=node.get("column_first", False),
            right_to_left=node.get("right_to_left", False),
            align=node.get("align"),
            title=node.get("title"),
        )
    if t == "layout":
        return build_layout(node["root"])
    return build_base(node)


def build_layout(node):
    layout = Layout(
        build(node["renderable"]) if "renderable" in node else None,
        name=node.get("name"),
        size=node.get("size"),
        minimum_size=node.get("minimum_size", 1),
        ratio=node.get("ratio", 1),
        visible=node.get("visible", True),
    )
    if "children" in node:
        children = [build_layout(child) for child in node["children"]]
        if node["splitter"] == "row":
            layout.split_row(*children)
        else:
            layout.split_column(*children)
    return layout


def render_live(case):
    node = case["renderable"]
    console = make_console(case, terminal=node.get("terminal", True))
    frames = [build(frame) for frame in node["frames"]]
    live = Live(
        frames[0],
        console=console,
        auto_refresh=False,
        transient=node.get("transient", False),
        vertical_overflow=node.get("vertical_overflow", "ellipsis"),
        redirect_stdout=False,
        redirect_stderr=False,
    )
    live.start(refresh=True)
    for frame in frames[1:]:
        live.update(frame, refresh=True)
    live.stop()
    return console.file.getvalue().encode("utf-8")


def render_case(case):
    if case["renderable"]["t"] == "live":
        return render_live(case)
    console = make_console(case)
    console.print(build(case["renderable"]))
    return console.file.getvalue().encode("utf-8")


def load_cases(path):
    with open(path) as f:
        return [json.loads(line) for line in f if line.strip()]


def render_case_isolated(case):
    out = subprocess.run([sys.executable, str(Path(__file__).resolve()), "--one"], input=json.dumps(case).encode(),
                         stdout=subprocess.PIPE, check=True)
    return out.stdout


def main():
    import argparse

    ap = argparse.ArgumentParser(description="Render corpus 3 with Python Rich")
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

    with ThreadPoolExecutor(max_workers=min(4, os.cpu_count() or 1)) as pool:
        list(pool.map(one, cases))
    print(f"{len(cases)} cases rendered into {args.outdir}, one fresh process per case")


if __name__ == "__main__":
    main()
