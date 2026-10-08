"""Speed harness: verify workload output, time S1 (whole process) and S2-S4 (in process), report medians with CIs.

Candidate protocol (every candidate ships these adapters; Python Rich ships them under reference/python/speed/):
  S1   <cmd s1>                                                   prints the small styled table, exits
  S2-4 <cmd bench> --workload S2|S3|S4 --input FILE [--emit | --warmup W --iterations N]
       --emit: render once to stdout. Timed mode: render N times (after W warm-ups) to stdout, one
       {"iter": i, "ns": elapsed} line per measured run on stderr. Input loading is not timed.

Environment for every speed run (SPEED_ENV): FORCE_COLOR=1 COLORTERM=truecolor TERM=xterm-256color COLUMNS=100,
so every workload reaches the styling and rendering path through a pipe.
"""

import argparse
import gzip
import json
import os
import subprocess
import sys
import threading
import time
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).parent))
from common import BENCH  # noqa: E402
from tasks import screen_snapshot, screen_text  # noqa: E402

SPEED_ENV = {"FORCE_COLOR": "1", "COLORTERM": "truecolor", "TERM": "xterm-256color", "COLUMNS": "100"}
INPUTS = {"S2": BENCH / "cases" / "speed" / "s2_table.tsv", "S3": None, "S4": BENCH / "cases" / "speed" / "s4_lines.txt"}
GOLDEN = BENCH / "golden" / "speed"
MIN_SAMPLES = 30
WARMUP = 5
RNG = np.random.default_rng(20261008)


def env():
    e = {"PATH": os.environ["PATH"], "HOME": os.environ.get("HOME", "/tmp"), "LANG": "en_US.UTF-8"}
    e.update(SPEED_ENV)
    return e


def workload_cmd(cmd, workload, extra):
    c = list(cmd) + ["--workload", workload]
    if INPUTS[workload]:
        c += ["--input", str(INPUTS[workload])]
    return c + extra


def run_capture(cmd, timeout=900):
    return subprocess.run(cmd, env=env(), stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                          stderr=subprocess.PIPE, timeout=timeout)


def golden_paths(workload):
    return {"S1": GOLDEN / "s1.bytes", "S2": GOLDEN / "s2.bytes.gz", "S3": GOLDEN / "s3.screen.txt",
            "S4": GOLDEN / "s4.bytes.gz"}[workload]


def normalize(workload, data: bytes):
    return screen_text(screen_snapshot(data)).encode() if workload == "S3" else data


def verify(workload, cmd):
    """Output must equal the golden (S3: final screen text) or the workload is discarded for this candidate."""
    argv = list(cmd) if workload == "S1" else workload_cmd(cmd, workload, ["--emit"])
    got = normalize(workload, run_capture(argv).stdout)
    gp = golden_paths(workload)
    want = gzip.decompress(gp.read_bytes()) if gp.suffix == ".gz" else gp.read_bytes()
    if workload == "S3":
        want = want.rstrip(b"\n")
    return got == want, len(got), len(want)


def bootstrap_ci(samples, stat=np.median, n=5000):
    s = np.asarray(samples, dtype=float)
    idx = RNG.integers(0, len(s), size=(n, len(s)))
    stats = stat(s[idx], axis=1)
    return float(stat(s)), float(np.percentile(stats, 2.5)), float(np.percentile(stats, 97.5))


def ratio_ci(a, b, n=5000):
    """Median(a)/median(b) with a 95% bootstrap CI (independent resampling of each sample set)."""
    a, b = np.asarray(a, float), np.asarray(b, float)
    ia = RNG.integers(0, len(a), size=(n, len(a)))
    ib = RNG.integers(0, len(b), size=(n, len(b)))
    r = np.median(a[ia], axis=1) / np.median(b[ib], axis=1)
    return float(np.median(a) / np.median(b)), float(np.percentile(r, 2.5)), float(np.percentile(r, 97.5))


def time_s1(cmd, runs=MIN_SAMPLES + WARMUP, warmup=WARMUP):
    first_byte, total = [], []
    for i in range(runs):
        t0 = time.perf_counter_ns()
        p = subprocess.Popen(cmd, env=env(), stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
        first = None
        while True:
            chunk = os.read(p.stdout.fileno(), 65536)
            if first is None and chunk:
                first = time.perf_counter_ns() - t0
            if not chunk:
                break
        p.wait()
        t1 = time.perf_counter_ns() - t0
        if i >= warmup:
            first_byte.append(first or t1)
            total.append(t1)
    return {"first_byte_ns": first_byte, "total_ns": total}


def time_inproc(cmd, workload, iterations=MIN_SAMPLES, warmup=WARMUP):
    argv = workload_cmd(cmd, workload, ["--warmup", str(warmup), "--iterations", str(iterations)])
    p = subprocess.Popen(argv, env=env(), stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    threading.Thread(target=lambda: [None for _ in iter(lambda: p.stdout.read(65536), b"")], daemon=True).start()
    ns = [json.loads(line)["ns"] for line in p.stderr if line.strip().startswith(b"{")]
    p.wait()
    return ns


def summarize(samples):
    med, lo, hi = bootstrap_ci(samples)
    return {"n": len(samples), "median_ms": med / 1e6, "ci95_ms": [lo / 1e6, hi / 1e6]}


def make_golden(py_cmd):
    GOLDEN.mkdir(parents=True, exist_ok=True)
    s1 = run_capture([*py_cmd[:1], str(BENCH / "reference/python/speed/s1.py")]).stdout
    golden_paths("S1").write_bytes(s1)
    bench = [py_cmd[0], str(BENCH / "reference/python/speed/bench.py")]
    for w in ("S2", "S4"):
        golden_paths(w).write_bytes(gzip.compress(run_capture(workload_cmd(bench, w, ["--emit"])).stdout, mtime=0))
    golden_paths("S3").write_text(screen_text(screen_snapshot(run_capture(workload_cmd(bench, "S3", ["--emit"])).stdout)) + "\n")
    print("speed goldens ->", GOLDEN)


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="mode", required=True)
    sub.add_parser("make-golden")
    v = sub.add_parser("verify")
    v.add_argument("workload", choices=["S1", "S2", "S3", "S4"])
    t = sub.add_parser("time")
    t.add_argument("workload", choices=["S1", "S2", "S3", "S4"])
    t.add_argument("--iterations", type=int, default=MIN_SAMPLES)
    t.add_argument("--out", type=Path)
    r = sub.add_parser("ratio", help="median ratio A/B with 95%% CI from two timing JSON files")
    r.add_argument("a", type=Path)
    r.add_argument("b", type=Path)
    r.add_argument("--key", default="ns")
    argv = sys.argv[1:]
    cmd = []
    if "--" in argv:
        cut = argv.index("--")
        argv, cmd = argv[:cut], argv[cut + 1:]
    args = ap.parse_args(argv)
    py = str(BENCH / ".venv" / "bin" / "python")
    if args.mode == "make-golden":
        make_golden([py])
        return
    if args.mode == "ratio":
        a, b = json.loads(args.a.read_text())[args.key], json.loads(args.b.read_text())[args.key]
        print("ratio A/B median %.3f  CI95 [%.3f, %.3f]" % ratio_ci(a, b))
        return
    if args.mode == "verify":
        ok, got, want = verify(args.workload, cmd)
        print(("EQUAL" if ok else "DIFFERENT"), args.workload, got, "bytes vs", want)
        raise SystemExit(0 if ok else 1)
    ok, got, want = verify(args.workload, cmd)
    if not ok:
        print(f"{args.workload}: output differs from the golden ({got} vs {want} bytes): workload discarded for this candidate")
        raise SystemExit(2)
    if args.workload == "S1":
        samples = time_s1(cmd, runs=args.iterations + WARMUP)
        result = {"first_byte_ns": samples["first_byte_ns"], "total_ns": samples["total_ns"]}
        print({k: summarize(v) for k, v in result.items()})
    else:
        ns = time_inproc(cmd, args.workload, args.iterations)
        result = {"ns": ns}
        print(summarize(ns))
    if args.iterations < MIN_SAMPLES:
        print(f"NOTE: {args.iterations} < {MIN_SAMPLES} samples: pipeline check, no verdict (evaluation.md, pilot validity 6)")
    if args.out:
        args.out.write_text(json.dumps({"workload": args.workload, **result}) + "\n")


if __name__ == "__main__":
    main()
