"""Adoption cost of a candidate: added clean-release compile time, stripped binary size, transitive dependencies.

Measured on the candidate's minimal `hello table` project against an empty program built the same way:
  adoption.py <candidate-project-dir> [--runs 3]
Method: `cargo fetch` first (network excluded), then for each run `cargo clean` + `cargo build --release
--offline` with strip = "symbols" forced identically for both projects; compile time is wall clock, binary size
the stripped release binary, dependencies the unique crates in `cargo tree -e normal` (self excluded).
The candidate's rustc and lockfile are used as they are; the baseline is rebuilt in the same session.
"""

import argparse
import json
import statistics
import subprocess
import sys
import tempfile
import time
from pathlib import Path

CFG = ["--config", 'profile.release.strip="symbols"']


def sh(cmd, cwd, check=True):
    return subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, check=check)


def package_name(project: Path):
    meta = json.loads(sh(["cargo", "metadata", "--no-deps", "--format-version", "1"], project).stdout)
    pkg = meta["packages"][0]
    bins = [t["name"] for t in pkg["targets"] if "bin" in t["kind"]]
    return pkg["name"], bins[0], Path(meta["target_directory"])


def measure(project: Path, runs: int):
    sh(["cargo", "fetch"], project)
    name, binname, target = package_name(project)
    times = []
    for _ in range(runs):
        sh(["cargo", "clean"], project)
        t0 = time.perf_counter()
        sh(["cargo", "build", "--release", "--offline", *CFG], project)
        times.append(time.perf_counter() - t0)
    size = (target / "release" / binname).stat().st_size
    tree = sh(["cargo", "tree", "-e", "normal", "--prefix", "none", "--format", "{p}"], project).stdout.split("\n")
    crates = {line.strip() for line in tree if line.strip()}
    return {"compile_s_median": statistics.median(times), "compile_s_runs": times, "binary_bytes": size,
            "transitive_deps": max(len(crates) - 1, 0)}


def baseline(runs: int):
    with tempfile.TemporaryDirectory() as d:
        p = Path(d) / "empty"
        sh(["cargo", "new", "--bin", "--vcs", "none", "--edition", "2024", str(p)], d)
        (p / "src" / "main.rs").write_text("fn main() {}\n")
        return measure(p, runs)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("project", type=Path)
    ap.add_argument("--runs", type=int, default=3)
    ap.add_argument("--out", type=Path)
    args = ap.parse_args()
    base = baseline(args.runs)
    cand = measure(args.project, args.runs)
    res = {
        "added_compile_s": cand["compile_s_median"] - base["compile_s_median"],
        "added_binary_bytes": cand["binary_bytes"] - base["binary_bytes"],
        "transitive_deps": cand["transitive_deps"],
        "candidate": cand, "baseline": base,
        "thresholds": {"added_compile_s": 15, "added_binary_bytes": 1_500_000, "transitive_deps": 60},
    }
    res["pass"] = (res["added_compile_s"] <= 15 and res["added_binary_bytes"] <= 1_500_000
                   and res["transitive_deps"] <= 60)
    print(json.dumps({k: v for k, v in res.items() if k not in ("candidate", "baseline")}, indent=1))
    if args.out:
        args.out.write_text(json.dumps(res, indent=1) + "\n")


if __name__ == "__main__":
    sys.exit(main())
