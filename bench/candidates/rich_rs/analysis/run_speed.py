"""Speed run for rich-rs with a quiet-machine gate (bench-design.md, pilot validity rule 1).

Usage: uv run python candidates/rich_rs/analysis/run_speed.py <out.json> [--iterations N] [--wait-seconds S]

For each workload S1-S4: verify the output against the golden (`speed.verify`), then time rich-rs and Python Rich
(reference/python/speed) one after another with the same protocol, and report medians, 95% bootstrap CIs and the
ratio of medians. A workload whose verify result is DIFFERENT is still timed here (speed.py would refuse), and is
labelled `verify: DIFFERENT`: the report must treat it as deviating work, comparable only with candidates that
show the same deviation (speed.md), never as a verdict number against Python Rich.

Quiet gate: before each workload the script waits until no other `cargo`, `rustc`, `cc`, `bench` or `speed.py`
process runs (its own children excluded); if the machine never goes quiet within --wait-seconds the workload is
timed anyway and marked `quiet: false` (pipeline check, no verdict).
"""

import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path

BENCH = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(BENCH / "scripts"))
import speed  # noqa: E402

T = BENCH / "candidates/rich_rs/tasks/target/release"
A = BENCH / "candidates/rich_rs/adapters/target/release"
PY = str(BENCH / ".venv/bin/python")
REF = BENCH / "reference/python/speed"
PATTERN = r"cargo (build|run|test|doc|rustdoc|check|clean|fetch|install)|rustc|target/release/(bench|s1|t0)|speed\.py|adoption\.py|docs_coverage|clang|/ld |cc1"


def others_running():
    me = {os.getpid(), os.getppid()}
    pids = subprocess.run(["pgrep", "-f", PATTERN], capture_output=True, text=True).stdout.split()
    rows = []
    for pid in pids:
        if int(pid) in me:
            continue
        cmd = subprocess.run(["ps", "-o", "command=", "-p", pid], capture_output=True, text=True).stdout.strip().splitlines()
        cmd = cmd[0] if cmd else ""
        if not cmd or "pgrep" in cmd or "run_speed.py" in cmd or "/bin/zsh -c" in cmd or cmd.startswith("claude "):
            continue
        rows.append(f"{pid} {cmd[:120]}")
    return rows


def wait_quiet(limit):
    start = time.time()
    while True:
        rows = others_running()
        if not rows:
            return True, time.time() - start, []
        if time.time() - start > limit:
            return False, time.time() - start, rows
        time.sleep(15)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("out", type=Path)
    ap.add_argument("--iterations", type=int, default=30)
    ap.add_argument("--wait-seconds", type=int, default=600)
    ap.add_argument("--only", nargs="*")
    ap.add_argument("--adoption", type=Path, help="also run adoption.py on the hello crate behind the quiet gate")
    args = ap.parse_args()
    cmds = {"S1": ([str(T / "s1")], [PY, str(REF / "s1.py")]),
            "S2": ([str(A / "bench")], [PY, str(REF / "bench.py")]),
            "S3": ([str(A / "bench")], [PY, str(REF / "bench.py")]),
            "S4": ([str(A / "bench")], [PY, str(REF / "bench.py")])}
    report = {"machine": subprocess.run("sysctl -n machdep.cpu.brand_string hw.memsize; sw_vers -productVersion; rustc -V; cargo -V",
                                        shell=True, capture_output=True, text=True).stdout.split("\n"), "workloads": {}}
    if args.adoption:
        for attempt in range(3):
            quiet, waited, rows = wait_quiet(args.wait_seconds)
            proc = subprocess.run(["uv", "run", "python", "scripts/adoption.py", "candidates/rich_rs/hello", "--out", str(args.adoption)],
                                  cwd=BENCH, capture_output=True, text=True)
            after = others_running()
            report["adoption"] = {"attempt": attempt + 1, "quiet_before": quiet, "waited_s": round(waited), "busy_with": rows,
                                  "busy_after": after, "clean": quiet and not after, "stdout": proc.stdout[-1500:], "stderr": proc.stderr[-300:]}
            print("adoption attempt", attempt + 1, "clean", report["adoption"]["clean"], proc.stdout[-300:], flush=True)
            if report["adoption"]["clean"]:
                break
        args.out.write_text(json.dumps(report) + "\n")
    for w in args.only or ["S1", "S2", "S3", "S4"]:
        cand, ref = cmds[w]
        attempts = []
        for attempt in range(3):
            quiet, waited, rows = wait_quiet(args.wait_seconds)
            ok, got, want = speed.verify(w, cand)
            entry = {"attempt": attempt + 1, "verify": "EQUAL" if ok else "DIFFERENT", "bytes": [got, want],
                     "quiet_before": quiet, "waited_s": round(waited), "busy_with": rows}
            series, clean = {}, quiet
            for name, cmd in (("rich_rs", cand), ("python_rich", ref)):
                if w == "S1":
                    series[name] = speed.time_s1(cmd, runs=args.iterations + speed.WARMUP)
                else:
                    series[name] = {"ns": speed.time_inproc(cmd, w, args.iterations)}
                after = others_running()
                entry.setdefault("busy_after_series", {})[name] = after
                clean = clean and not after
            entry["clean"] = clean
            entry["samples"] = series
            keys = ["first_byte_ns", "total_ns"] if w == "S1" else ["ns"]
            entry["summary"] = {}
            for key in keys:
                a, b = series["rich_rs"][key], series["python_rich"][key]
                r = speed.ratio_ci(a, b)
                entry["summary"][key] = {"rich_rs": speed.summarize(a), "python_rich": speed.summarize(b),
                                         "ratio_rich_rs_over_python": {"median": r[0], "ci95": [r[1], r[2]]}}
            attempts.append(entry)
            print(w, "attempt", attempt + 1, entry["verify"], "clean", clean, json.dumps(entry["summary"])[:300], flush=True)
            if clean:
                break
        report["workloads"][w] = {"final": attempts[-1], "attempts": len(attempts)}
        args.out.write_text(json.dumps(report) + "\n")


if __name__ == "__main__":
    main()
