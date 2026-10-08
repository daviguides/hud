"""In-process speed workloads S2-S4 for Python Rich, following the candidate protocol.

  bench.py --workload S2|S3|S4 --input FILE [--emit | --warmup W --iterations N]

--emit        run once, write the rendered output to stdout (verification)
--iterations  after W untimed warm-up runs, write the output to stdout N times and print one
              {"iter": i, "ns": elapsed} JSON line per measured run to stderr. The timed region
              covers build + render + write + flush; input loading is excluded.
"""

import argparse
import json
import sys
import time

from rich.console import Console
from rich.progress import BarColumn, MofNCompleteColumn, Progress, TaskProgressColumn, TextColumn
from rich.table import Table

STATUS_COLOR = {"ok": "green", "warn": "yellow", "fail": "red", "skip": "dim"}


def console():
    return Console(highlight=False)


def s2(rows):
    table = Table(title="Results", header_style="bold cyan")
    for name, justify in (("Id", "right"), ("Name", "left"), ("Status", "left"), ("Value", "right"), ("Note", "left")):
        table.add_column(name, justify=justify)
    for r in rows:
        table.add_row(r[0], r[1], f"[{STATUS_COLOR[r[2]]}]{r[2]}[/]", r[3], r[4])
    console().print(table)


def s3(_):
    tasks_n, total_updates, per_task = 8, 100_000, 12_500
    progress = Progress(
        TextColumn("{task.description}"),
        BarColumn(bar_width=30),
        TaskProgressColumn(),
        MofNCompleteColumn(),
        console=console(),
        auto_refresh=False,
    )
    with progress:
        ids = [progress.add_task(f"task {i}", total=per_task) for i in range(tasks_n)]
        for k in range(total_updates):
            progress.advance(ids[k % tasks_n])
            if k % 100 == 99:
                progress.refresh()


def s4(lines):
    c = console()
    for line in lines:
        c.print(line)


def load(workload, path):
    if workload == "S2":
        with open(path) as f:
            return [ln.rstrip("\n").split("\t") for ln in f if ln.strip()]
    if workload == "S4":
        with open(path) as f:
            return [ln.rstrip("\n") for ln in f]
    return None


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--workload", required=True, choices=["S2", "S3", "S4"])
    ap.add_argument("--input")
    ap.add_argument("--emit", action="store_true")
    ap.add_argument("--warmup", type=int, default=5)
    ap.add_argument("--iterations", type=int, default=30)
    args = ap.parse_args()
    fn = {"S2": s2, "S3": s3, "S4": s4}[args.workload]
    data = load(args.workload, args.input)
    if args.emit:
        fn(data)
        sys.stdout.flush()
        return
    for _ in range(args.warmup):
        fn(data)
        sys.stdout.flush()
    for i in range(args.iterations):
        t0 = time.perf_counter_ns()
        fn(data)
        sys.stdout.flush()
        ns = time.perf_counter_ns() - t0
        print(json.dumps({"iter": i, "ns": ns}), file=sys.stderr, flush=True)


if __name__ == "__main__":
    main()
