"""Uniform pilot driver: runs every candidate through the same corrected harness.

  pilot.py static [candidate ...]     correctness, width, capability, tasks, LOC, API surface (deterministic axes)
  pilot.py timed  [candidate ...]     adoption cost and speed S1-S4 for each candidate and Python Rich, sequentially,
                                       only while the machine is idle (guard below); every run records the load
  pilot.py speed WORKLOADS [candidate ...]   only the speed part, for the listed workloads (e.g. S3 or S1,S2,S3,S4)
  pilot.py unverified [candidate ...]  times the workloads whose output differs from the golden, into
                                       <W>.unverified.json (different work: informational, never ranked)

Per-candidate invocation details (adapter paths, argument order) are data in CANDIDATES; the measurement code is
shared. Results land in `pilot/<candidate>/` (committed) and raw outputs in `results/<candidate>/` (ignored).
Nothing here changes a threshold: `evaluation.md` is the only source of gates.
"""

import json
import os
import re
import statistics
import subprocess
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import api_surface  # noqa: E402
import loc  # noqa: E402
from common import BENCH  # noqa: E402

PY = str(BENCH / ".venv" / "bin" / "python")
TASK_IDS = ["t01-table", "t02-panel", "t03-progress", "t04-tree", "t05-error", "t06-markup", "t07-pipe", "t08-env"]
CASES = str(BENCH / "cases" / "correctness.jsonl")


def c(rel):
    return str(BENCH / rel)


def seq(prefix, names, ext=""):
    return {tid: f"{prefix}/{n}{ext}" for tid, n in zip(TASK_IDS, names)}


T = [f"t0{i}" for i in range(1, 9)]
RS_RICH_T = ["t01_table", "t02_panel", "t03_progress", "t04_tree", "t05_error", "t06_markup", "t07_pipe", "t08_env"]

CANDIDATES = {
    "rich_rust": {
        "root": "candidates/rich_rust", "crates": ["rich_rust"], "hello": ["hello"],
        "cases": lambda out: [c("candidates/rich_rust/adapters/target/release/cases-runner"), CASES, out],
        "width": lambda out: [c("candidates/rich_rust/adapters/target/release/width-runner"), out, str(BENCH)],
        "cap": c("candidates/rich_rust/adapters/target/release/cap"),
        "tasks": seq(c("candidates/rich_rust/solutions/target/release"), T),
        "loc": seq("candidates/rich_rust/solutions/src/bin", T, ".rs"),
        "s1": [c("candidates/rich_rust/adapters/target/release/s1")],
        "bench": [c("candidates/rich_rust/adapters/target/release/bench")],
    },
    "rs_rich": {
        "root": "candidates/rs_rich", "crates": ["rs-rich"], "hello": ["hello", "hello_nodefault"],
        "cases": lambda out: [c("candidates/rs_rich/target/release/cases_runner"), CASES, out],
        "width": lambda out: [c("candidates/rs_rich/target/release/width_runner"), out],
        "cap": c("candidates/rs_rich/target/release/cap"),
        "tasks": seq(c("candidates/rs_rich/target/release"), RS_RICH_T),
        "loc": seq("candidates/rs_rich/src/bin", RS_RICH_T, ".rs"),
        "s1": [c("candidates/rs_rich/target/release/s1")],
        "bench": [c("candidates/rs_rich/target/release/bench")],
    },
    "richrs": {
        "root": "candidates/richrs", "crates": ["richrs"], "hello": ["hello"],
        "cases": lambda out: [c("candidates/richrs/target/release/cases_runner"), CASES, out],
        "width": lambda out: [c("candidates/richrs/target/release/width_runner"), out],
        "cap": c("candidates/richrs/target/release/cap"),
        "tasks": seq(c("candidates/richrs/target/release"), T),
        "loc": seq("candidates/richrs/src/bin", T, ".rs"),
        "s1": [c("candidates/richrs/target/release/s1")],
        "bench": [c("candidates/richrs/target/release/bench")],
    },
    "rich_rs": {
        "root": "candidates/rich_rs", "crates": ["rich-rs"], "hello": ["hello"],
        "cases": lambda out: [c("candidates/rich_rs/adapters/target/release/cases-runner"), CASES, out],
        "width": lambda out: [c("candidates/rich_rs/adapters/target/release/width-runner"), out],
        "cap": c("candidates/rich_rs/tasks/target/release/cap"),
        "tasks": seq(c("candidates/rich_rs/tasks/target/release"), T),
        "loc": seq("candidates/rich_rs/tasks/src/bin", T, ".rs"),
        "s1": [c("candidates/rich_rs/tasks/target/release/s1")],
        "bench": [c("candidates/rich_rs/adapters/target/release/bench")],
    },
    "composed": {
        "root": "candidates/composed", "crates": ["comfy-table", "owo-colors", "indicatif"], "hello": ["hello"],
        "cases": lambda out: [c("candidates/composed/target/release/cases-runner"), CASES, out],
        "width": lambda out: [c("candidates/composed/target/release/width-runner"), str(BENCH / "cases"), out],
        "cap": c("candidates/composed/target/release/cap"),
        "tasks": seq(c("candidates/composed/target/release"), T),
        "loc": seq("candidates/composed/src/bin", T, ".rs"),
        "s1": [c("candidates/composed/target/release/s1")],
        "bench": [c("candidates/composed/target/release/bench")],
    },
}


def run(cmd, **kw):
    return subprocess.run(cmd, cwd=BENCH, capture_output=True, text=True, **kw)


def out_dirs(name):
    raw, pilot = BENCH / "results" / name, BENCH / "pilot" / name
    raw.mkdir(parents=True, exist_ok=True)
    pilot.mkdir(parents=True, exist_ok=True)
    return raw, pilot


def kept_classifications(name):
    """Only `documented_deviation` entries survive: `reference_quirk` was a harness defect (goldens are now one
    fresh process per case) and `candidate_bug` is the default for any difference."""
    keep = {}
    for p in (BENCH / "candidates" / name).rglob("classifications*.json"):
        if "/target/" in str(p):
            continue
        for cid, v in json.loads(p.read_text()).items():
            if isinstance(v, dict) and v.get("class") == "documented_deviation":
                keep[cid] = v
    return keep


def static(name):
    cfg = CANDIDATES[name]
    raw, pilot = out_dirs(name)
    # 1 correctness
    cdir = raw / "correctness"
    if cdir.exists():
        subprocess.run(["rm", "-rf", str(cdir)])
    cdir.mkdir(parents=True)
    r = run(cfg["cases"](str(cdir)))
    (pilot / "cases_runner.log").write_text((r.stdout + r.stderr)[-2000:])
    keep = kept_classifications(name)
    if keep:
        (cdir / "classifications.json").write_text(json.dumps(keep, indent=1))
    sup = BENCH / cfg["root"] / "support.json"
    r = run([PY, "scripts/compare.py", str(cdir), "--out", str(pilot / "correctness.json"), "--show", "0"])
    (pilot / "correctness.txt").write_text(r.stdout + r.stderr)
    # 2 width + tables
    wdir = raw / "width"
    if wdir.exists():
        subprocess.run(["rm", "-rf", str(wdir)])
    wdir.mkdir(parents=True)
    r = run(cfg["width"](str(wdir)))
    (pilot / "width_runner.log").write_text((r.stdout + r.stderr)[-2000:])
    r = run([PY, "scripts/width_check.py", str(wdir), "--out", str(pilot / "width.json")])
    (pilot / "width.txt").write_text(r.stdout + r.stderr)
    # capability
    r = run([PY, "scripts/capability.py", "check", "--out", str(pilot / "capability.json"), "--", cfg["cap"]])
    (pilot / "capability.txt").write_text(r.stdout + r.stderr)
    # tasks + loc
    results, locs = {}, {}
    for tid in TASK_IDS:
        r = run([PY, "scripts/tasks.py", "check", tid, "--", cfg["tasks"][tid]])
        runs = re.findall(r"^(PASS|FAIL) " + re.escape(tid) + r"/(\S+)", r.stdout, re.M)
        results[tid] = {name_: status == "PASS" for status, name_ in runs}
        locs[tid] = loc.count(BENCH / cfg["loc"][tid])
    passing = [t for t in TASK_IDS if results[t] and all(results[t].values())]
    base = json.loads((BENCH / "golden" / "tasks" / "python_loc.json").read_text())
    py = [base[t.split("-")[0]] for t in TASK_IDS]
    med = statistics.median([locs[t] for t in passing]) if passing else None
    (pilot / "tasks.json").write_text(json.dumps({
        "runs": results, "tasks_passing": len(passing), "runs_passing": sum(sum(v.values()) for v in results.values()),
        "runs_total": sum(len(v) for v in results.values()), "loc": locs, "loc_median_passing": med,
        "python_loc_median": statistics.median(py), "loc_ratio_vs_python": (med / statistics.median(py)) if med else None,
    }, indent=1) + "\n")
    # API surface (union across the candidate's crates)
    parity_union, padded, friction_totals = {}, [], {"public_functions": 0, "more_than_3_positional": 0,
                                                       "with_option_params": 0, "none_padding": 0}
    for crate in cfg["crates"]:
        r = run([PY, "scripts/api_surface.py", "json", str(BENCH / cfg["root"] / cfg["hello"][0]), crate])
        path = r.stdout.strip().splitlines()[-1]
        doc = api_surface.load(path)
        for n, ok in api_surface.parity(doc).items():
            parity_union[n] = parity_union.get(n, False) or ok
        f = api_surface.friction(doc)
        friction_totals["public_functions"] += f["public_functions"]
        friction_totals["more_than_3_positional"] += len(f["more_than_3_positional"])
        friction_totals["with_option_params"] += len(f["with_option_params"])
        friction_totals["none_padding"] += len(f["none_padding_candidates"])
        padded += [f"{crate}:{x['owner']}::{x['name']}" for x in f["none_padding_candidates"]]
    hit = sum(parity_union.values())
    (pilot / "api.json").write_text(json.dumps({
        "parity_hit": hit, "parity_total": len(parity_union), "parity": hit / len(parity_union),
        "missing": [n for n, ok in parity_union.items() if not ok], "friction": friction_totals,
        "none_padding_candidates": padded}, indent=1) + "\n")
    print("static done:", name)


def busy():
    r = subprocess.run(["pgrep", "-x", "cargo|rustc|rustdoc|cc|ld|clang|rustfmt"], capture_output=True, text=True)
    return r.stdout.split()


def load1():
    return os.getloadavg()[0]


def wait_idle(log, label, max_load=4.0, patience=1800):
    """Wait for no cargo/rustc-family process and 1 minute load average <= max_load; record the state at start."""
    t0 = time.time()
    while busy() or load1() > max_load:
        if time.time() - t0 > patience:
            log.append(f"{label}: PROCEEDED UNDER LOAD after {patience}s (load {load1():.2f}, busy {busy()})")
            break
        time.sleep(5)
    log.append(f"{label}: start load1={load1():.2f} busy={busy()}")


def speed_cmd(name, workload):
    if name == "python":
        return [PY, str(BENCH / "reference/python/speed" / ("s1.py" if workload == "S1" else "bench.py"))]
    cfg = CANDIDATES[name]
    return cfg["s1"] if workload == "S1" else cfg["bench"]


def timed(names, adoption=True, workloads=("S1", "S2", "S3", "S4")):
    log = []
    for name in names if adoption else []:
        raw, pilot = out_dirs(name)
        sp = pilot / "speed"
        sp.mkdir(exist_ok=True)
        cfg = CANDIDATES[name]
        for variant, hello in enumerate(cfg["hello"]):
            wait_idle(log, f"{name} adoption {hello}")
            r = run([PY, "scripts/adoption.py", str(BENCH / cfg["root"] / hello), "--out", str(pilot / f"adoption_{hello}.json")])
            log.append(f"{name} adoption {hello}: end load1={load1():.2f} busy={busy()}")
            (pilot / f"adoption_{hello}.txt").write_text(r.stdout + r.stderr)
    for workload in workloads:
        for name in names + ["python"]:
            cmd = speed_cmd(name, workload)
            sp = BENCH / "pilot" / name / "speed"
            sp.mkdir(parents=True, exist_ok=True)
            wait_idle(log, f"{name} {workload}")
            v = run([PY, "scripts/speed.py", "verify", workload, "--"] + cmd)
            t = run([PY, "scripts/speed.py", "time", workload, "--iterations", "30", "--out", str(sp / f"{workload}.json"), "--"] + cmd)
            log.append(f"{name} {workload}: verify={v.stdout.strip()!r} end load1={load1():.2f} busy={busy()}")
            (sp / f"{workload}.txt").write_text(v.stdout + v.stderr + t.stdout + t.stderr)
    logfile = BENCH / "pilot" / ("speed_conditions.log" if adoption else "speed_conditions_" + "_".join(workloads) + ".log")
    logfile.write_text("\n".join(log) + "\n")
    print("\n".join(log))


def unverified(names):
    log = []
    for workload in ("S1", "S2", "S4"):
        for name in names:
            if (BENCH / "pilot" / name / "speed" / f"{workload}.json").exists():
                continue
            cmd = speed_cmd(name, workload)
            sp = BENCH / "pilot" / name / "speed"
            wait_idle(log, f"{name} {workload} (unverified)")
            run([PY, "scripts/speed.py", "time", workload, "--iterations", "30", "--unverified", "--out",
                 str(sp / f"{workload}.unverified.json"), "--"] + cmd)
            log.append(f"{name} {workload} unverified: end load1={load1():.2f} busy={busy()}")
    (BENCH / "pilot" / "speed_conditions_unverified.log").write_text("\n".join(log) + "\n")
    print("\n".join(log))


def main():
    mode, names = sys.argv[1], sys.argv[2:]
    names = names or ([] if mode == "speed" else list(CANDIDATES))
    if mode == "static":
        for n in names:
            static(n)
    elif mode == "timed":
        timed(names)
    elif mode == "unverified":
        unverified(names or list(CANDIDATES))
    elif mode == "speed":
        workloads = tuple(names[0].split(","))
        timed(names[1:] or list(CANDIDATES), adoption=False, workloads=workloads)
    else:
        raise SystemExit(__doc__)


if __name__ == "__main__":
    main()
