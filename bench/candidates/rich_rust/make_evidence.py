"""Collect the committed evidence for the rich_rust candidate from the git-ignored results/ directory.

  uv run python candidates/rich_rust/make_evidence.py        (from bench/)
Writes candidates/rich_rust/evidence/: runs.jsonl (one classified record per case, run and cell), speed_summary.json,
machine.txt and copies of the raw reports.
"""
import json
import shutil
import subprocess
import sys
from pathlib import Path

BENCH = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(BENCH / "scripts"))
import speed  # noqa: E402

RES = BENCH / "results" / "rich_rust"
OUT = Path(__file__).resolve().parent / "evidence"
OUT.mkdir(exist_ok=True)


def sh(cmd):
    return subprocess.run(cmd, shell=True, capture_output=True, text=True).stdout.strip()


runs = []
cls = json.loads((RES / "correctness" / "classifications.json").read_text())
report = json.loads((RES / "correctness.json").read_text())
failed = {f["id"] for f in report["failures"]}
for line in (BENCH / "cases" / "correctness.jsonl").read_text().splitlines():
    case = json.loads(line)
    i = case["id"]
    rec = {"kind": "correctness", "id": i, "feature": case["feature"], "color_system": case["color_system"]}
    if i not in failed:
        rec["result"] = "pass"
    else:
        rec.update(result="fail" if cls[i]["class"] != "reference_quirk" else "excluded", **cls[i])
    runs.append(rec)
for line in (RES / "tasks.txt").read_text().splitlines():
    status, name = line.split(" ", 1)
    rec = {"kind": "task", "id": name, "result": "pass" if status == "PASS" else "fail"}
    if status == "FAIL":
        rec.update({"class": "candidate_failure",
                    "reason": "NO_COLOR=1 on a terminal: the crate drops every SGR attribute including bold (color_system None), the target keeps bold"})
    runs.append(rec)
for cell in json.loads((RES / "capability.json").read_text()):
    rec = {"kind": "capability", "id": cell["id"], "result": "pass" if cell["ok"] else "fail"}
    if not cell["ok"]:
        rec.update({"class": "candidate_failure", "want": cell["want"], "got": cell["got"], "note": "; ".join(cell["notes"])})
    runs.append(rec)
(OUT / "runs.jsonl").write_text("".join(json.dumps(r, ensure_ascii=False) + "\n" for r in runs))

summary = {}
for session in ("speed", "speed_run1"):
    d = RES / session
    summary[session] = {}
    for f in sorted(d.glob("*-S*.json")):
        data = json.loads(f.read_text())
        entry = {k: speed.summarize(v) for k, v in data.items() if isinstance(v, list)}
        summary[session][f.stem] = entry
(OUT / "speed_summary.json").write_text(json.dumps(summary, indent=1) + "\n")

machine = [
    "cpu: " + sh("sysctl -n machdep.cpu.brand_string"),
    "memory_bytes: " + sh("sysctl -n hw.memsize"),
    "os: " + sh("sw_vers | tr '\\n' ' '"),
    "rustc: " + sh("rustc -V"),
    "cargo: " + sh("cargo -V"),
    "rich_rust: =0.2.3 (crates.io), unicode-width 0.2.2, crossterm 0.29.0",
    "python reference: " + sh(f"cd {BENCH} && uv run python -c 'import rich,sys;print(sys.version.split()[0], \"rich\", __import__(\"importlib.metadata\").metadata.version(\"rich\"))'"),
    "generated: " + sh("date '+%Y-%m-%d %H:%M:%S %Z'"),
]
(OUT / "machine.txt").write_text("\n".join(machine) + "\n")

for name in ("correctness.json", "correctness.txt", "width.json", "capability.json", "adoption.json", "name_parity.txt",
             "api_friction.txt", "tasks.txt", "loc.txt", "golden_pollution.json"):
    shutil.copy(RES / name, OUT / name)
shutil.copy(RES / "correctness" / "classifications.json", OUT / "classifications.json")
for session in ("speed", "speed_run1"):
    for name in ("run.log", "load.log"):
        shutil.copy(RES / session / name, OUT / f"{session}.{name}")
print("evidence ->", OUT, len(runs), "records")
