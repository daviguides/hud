"""Speed A/B: hud (this checkout, "new") against an older build ("old", a worktree of a tag), alternating. Criterion S10 of v0.8
(new = v0.8, old = v0.7.0) and criterion 4 of status-criteria.md (new = v0.9, old = v0.8.0).

  ab_speed.py --old DIR_OF_V07_RELEASE_BINARIES [--rounds 4] [--out DIR]

For S1 to S4: both builds must produce the golden output (speed.py verify); then ROUNDS rounds, each timing the old build
and then the new one in the same session (30 measured samples after 5 warmup per build per round), a round starting only
when the 1 minute load average is at most 4.0 and no cargo or rustc runs (waiting up to 15 minutes; otherwise the round
runs anyway and is flagged `under_load`, which makes the verdict INCONCLUSIVE). Reports median(new)/median(old) with a
95% bootstrap CI (speed.ratio_ci). Also times JSON mode of S2 (`HUD_FORMAT=json`, same binary) against rich mode of the
same input in the same rounds. Nothing here changes a threshold: S10 is `foundation/structured-output-criteria.md`.
"""

import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import speed  # noqa: E402
from common import BENCH  # noqa: E402

NEW = BENCH / "candidates" / "hud" / "target" / "release"
INPUTS = {"S2": BENCH / "cases" / "speed" / "s2_table.tsv", "S3": None, "S4": BENCH / "cases" / "speed" / "s4_lines.txt"}


def idle(log, label, max_load=4.0, patience=900):
    t0 = time.time()
    while True:
        busy = subprocess.run(["pgrep", "-x", "cargo|rustc|rustdoc|cc|ld|clang|rustfmt"], capture_output=True, text=True).stdout.split()
        load = os.getloadavg()[0]
        if not busy and load <= max_load:
            log.append({"round": label, "load1": round(load, 2), "under_load": False})
            return False
        if time.time() - t0 > patience:
            log.append({"round": label, "load1": round(load, 2), "under_load": True, "busy": busy})
            return True
        time.sleep(15)


def cmd(bindir, workload):
    return [str(bindir / "s1_table")] if workload == "S1" else [str(bindir / "bench")]


def sample(bindir, workload, env_extra=None):
    original = speed.env
    if env_extra:
        speed.env = lambda: {**original(), **env_extra}
    try:
        if workload == "S1":
            return speed.time_s1(cmd(bindir, workload))["first_byte_ns"]
        argv = cmd(bindir, workload)
        if workload in INPUTS and INPUTS[workload]:
            argv = argv  # the input flag is added by workload_cmd
        return speed.time_inproc(argv, workload)
    finally:
        speed.env = original


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--old", required=True)
    ap.add_argument("--rounds", type=int, default=4)
    ap.add_argument("--out", default=str(BENCH / "pilot" / "hud" / "ab_v07_v08"))
    args = ap.parse_args()
    old = Path(args.old)
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    result, log = {}, []
    for workload in ("S1", "S2", "S3", "S4"):
        ok = []
        for name, bindir in (("old", old), ("new", NEW)):
            good, got, want, note = speed.verify(workload, cmd(bindir, workload))
            ok.append((name, good, note))
        if not all(g for _, g, _ in ok):
            result[workload] = {"verified": ok}
            continue
        old_s, new_s, flags = [], [], []
        for r in range(args.rounds):
            flags.append(idle(log, f"{workload}-{r}"))
            old_s += sample(old, workload)
            new_s += sample(NEW, workload)
        ratio, lo, hi = speed.ratio_ci(new_s, old_s)
        result[workload] = {"verified": ok, "n_old": len(old_s), "n_new": len(new_s), "median_old_ms": sorted(old_s)[len(old_s) // 2] / 1e6,
                            "median_new_ms": sorted(new_s)[len(new_s) // 2] / 1e6, "ratio_new_over_old": ratio, "ci95": [lo, hi],
                            "under_load": any(flags)}
        (out / f"{workload}_old.json").write_text(json.dumps(old_s))
        (out / f"{workload}_new.json").write_text(json.dumps(new_s))
    # JSON mode against rich mode of S2, same binary, alternating
    rich, js, flags = [], [], []
    for r in range(args.rounds):
        flags.append(idle(log, f"S2json-{r}"))
        rich += sample(NEW, "S2")
        js += sample(NEW, "S2", {"HUD_FORMAT": "json"})
    ratio, lo, hi = speed.ratio_ci(js, rich)
    result["S2_json_over_rich"] = {"n_rich": len(rich), "n_json": len(js), "median_rich_ms": sorted(rich)[len(rich) // 2] / 1e6,
                                   "median_json_ms": sorted(js)[len(js) // 2] / 1e6, "ratio_json_over_rich": ratio, "ci95": [lo, hi],
                                   "under_load": any(flags)}
    result["load_log"] = log
    (out / "summary.json").write_text(json.dumps(result, indent=1) + "\n")
    lines = []
    for w in ("S1", "S2", "S3", "S4"):
        r = result[w]
        if "ratio_new_over_old" in r:
            lines.append(f"{w}: new/old = {r['ratio_new_over_old']:.3f} (95% CI {r['ci95'][0]:.3f} to {r['ci95'][1]:.3f}); "
                         f"median {r['median_old_ms']:.3f} ms -> {r['median_new_ms']:.3f} ms; under_load={r['under_load']}")
        else:
            lines.append(f"{w}: output not verified {r['verified']}")
    j = result["S2_json_over_rich"]
    lines.append(f"S2 JSON/rich = {j['ratio_json_over_rich']:.3f} (95% CI {j['ci95'][0]:.3f} to {j['ci95'][1]:.3f}); "
                 f"rich {j['median_rich_ms']:.2f} ms, json {j['median_json_ms']:.2f} ms; under_load={j['under_load']}")
    lines.append("loads: " + ", ".join(f"{e['round']}={e['load1']}" for e in log))
    (out / "summary.txt").write_text("\n".join(lines) + "\n")
    print("\n".join(lines))


if __name__ == "__main__":
    main()
