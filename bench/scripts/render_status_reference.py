"""Reference renderer for corpus 5 (Spinner, Status): case -> ANSI bytes with pinned Python Rich.

The clock is injected (`Console(get_time=...)`) and the refresh thread of a Status is switched off
(`Status._live.auto_refresh = False`), so every frame is drawn by an explicit event at a stated
time. One fresh process per case (Rich caches a Style's escape codes on first use whatever the
color system, so a shared process would make bytes depend on the order of cases).
"""

import io
import json
import os
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

from rich.console import Console
from rich.spinner import Spinner
from rich.status import Status

sys.path.insert(0, str(Path(__file__).resolve().parent))
from render_reference import COLOR_SYSTEMS  # noqa: E402


def make_console(case, clock, terminal=True):
    return Console(
        file=io.StringIO(),
        width=case["width"],
        height=case.get("height", 24),
        force_terminal=terminal,
        color_system=COLOR_SYSTEMS[case["color_system"]],
        legacy_windows=False,
        emoji=False,
        highlight=False,
        _environ={},
        get_time=lambda: clock[0],
    )


def render_spinner(case):
    node = case["renderable"]
    clock = [0.0]
    console = make_console(case, clock)
    spinner = Spinner(node["name"], text=node["text"] or "", style=node["style"], speed=node["speed"])
    for op in node["ops"]:
        if op["op"] == "print":
            clock[0] = op["at"]
            console.print(spinner)
        elif op["op"] == "update":
            spinner.update(text=op["text"], style=op["style"], speed=op["speed"])
    return console.file.getvalue().encode("utf-8")


def render_status(case):
    node = case["renderable"]
    clock = [0.0]
    console = make_console(case, clock, terminal=node.get("terminal", True))
    status = Status(
        node["text"],
        console=console,
        spinner=node["spinner"],
        spinner_style=node["spinner_style"] or "status.spinner",
        speed=node["speed"],
    )
    status._live.auto_refresh = False
    for event in node["events"]:
        clock[0] = event["at"]
        op = event["op"]
        if op == "start":
            status.start()
        elif op == "refresh":
            status._live.refresh()
        elif op == "update":
            kwargs = {}
            if "text" in event:
                kwargs["status"] = event["text"]
            if "spinner" in event:
                kwargs["spinner"] = event["spinner"]
            if "spinner_style" in event:
                kwargs["spinner_style"] = event["spinner_style"]
            if "speed" in event:
                kwargs["speed"] = event["speed"]
            status.update(**kwargs)
        elif op == "stop":
            status.stop()
    return console.file.getvalue().encode("utf-8")


def render_case(case):
    if case["renderable"]["t"] == "spinner":
        return render_spinner(case)
    return render_status(case)


def load_cases(path):
    with open(path) as f:
        return [json.loads(line) for line in f if line.strip()]


def render_case_isolated(case):
    out = subprocess.run([sys.executable, str(Path(__file__).resolve()), "--one"], input=json.dumps(case).encode(),
                         stdout=subprocess.PIPE, check=True)
    return out.stdout


def main():
    import argparse

    ap = argparse.ArgumentParser(description="Render corpus 5 with Python Rich")
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
