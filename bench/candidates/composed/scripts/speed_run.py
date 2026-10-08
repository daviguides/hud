"""Run the shared speed harness functions for the composed candidate and for Python Rich in the same
session, recording whether the output matched the golden (verified) and whether the machine was quiet
(no cargo/rustc process) before and after each measurement. Uses speed.py's functions unchanged; the
only difference from `speed.py time` is that an unverified workload is timed and flagged instead of
refused, so the number is informational (bench-design.md, pilot validity 2)."""

import json
import subprocess
import sys
from pathlib import Path

BENCH = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(BENCH / "scripts"))
import speed  # noqa: E402

PY = str(BENCH / ".venv/bin/python")
COMPOSED = BENCH / "candidates/composed/target/release"
OUT = BENCH / "results/composed/speed"


def busy():
    out = subprocess.run(["pgrep", "-lx", "cargo|rustc|rustdoc|ld"], capture_output=True, text=True).stdout.strip()
    return out.splitlines()


def one(workload, label, cmd):
    verified, got, want = speed.verify(workload, cmd)
    before = busy()
    if workload == "S1":
        s = speed.time_s1(cmd, runs=speed.MIN_SAMPLES + speed.WARMUP)
        res = {"first_byte_ns": s["first_byte_ns"], "total_ns": s["total_ns"]}
    else:
        res = {"ns": speed.time_inproc(cmd, workload, speed.MIN_SAMPLES)}
    after = busy()
    res.update(workload=workload, label=label, verified=verified, quiet_before=not before, quiet_after=not after)
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / f"{workload}_{label}.json").write_text(json.dumps(res) + "\n")
    summary = {k: speed.summarize(v) for k, v in res.items() if k.endswith("ns")}
    print(workload, label, "verified" if verified else "UNVERIFIED", "quiet" if not before and not after else f"BUSY {before} {after}", summary, flush=True)


def main():
    py_s1 = [PY, str(BENCH / "reference/python/speed/s1.py")]
    py_bench = [PY, str(BENCH / "reference/python/speed/bench.py")]
    which = sys.argv[1:] or ["S1", "S2", "S3", "S4"]
    for w in which:
        one(w, "python_rich", py_s1 if w == "S1" else py_bench)
        one(w, "composed", [str(COMPOSED / "s1")] if w == "S1" else [str(COMPOSED / "bench")])


if __name__ == "__main__":
    main()
