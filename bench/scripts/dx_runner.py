"""DX first-try runner (bench-design.md, "First-try protocol"): fresh agent per run, one attempt, scored by the harness.

  dx_runner.py estimate [--candidate hud]
  dx_runner.py run --dry-run [--candidate hud] [--tasks t03-progress ...] [--repeats N] [--mock solution|broken|wrong|peek]
  dx_runner.py run --execute --primary-model ID [--stress-model ID] [--yes]   (spends tokens; see below)
  dx_runner.py report runs.jsonl

Per run: a run directory whose only content is a symlink to the docs mirror and to the README (scripts/docs_mirror.py);
the prompt of spec/dx-prompt.md; an agent that may use Read, Grep and Glob and nothing else; its reply, whose last rust
block is the program; a sandbox crate where the program is built once (`cargo build --locked --offline`); every run
of the task checked with scripts/tasks.py. The transcript is audited: a tool other than Read, Grep or Glob, or a path
outside the mirror, is a protocol violation and the run is discarded and counts for nothing until rerun.

Classification: success; `candidate_failure` (no code, compile error, wrong output, literal output); `harness_error`
(sandbox or mirror problem: excluded, rerun); `protocol_violation` (excluded, rerun); `unsupported` (the candidate
has no solution to give the mock for the task). A real run needs `--execute`, both model ids pinned in
spec/dx-models.json or given on the command line, and prints the token and cost estimate first (`--yes` skips the
question). The dry run and every test use the mock agent: no model is called.
"""

import argparse
import json
import os
import random
import re
import shutil
import statistics
import subprocess
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from common import BENCH  # noqa: E402

SPEC = BENCH / "spec"
TASK_SPEC = json.loads((SPEC / "tasks.json").read_text())["tasks"]
ALL_TASKS = [t["id"] for t in TASK_SPEC]
PYTHON = str(BENCH / ".venv" / "bin" / "python")
ALLOWED_TOOLS = {"Read", "Grep", "Glob"}
BOOTSTRAP = 5000
RUST_BLOCK = re.compile(r"```rust\s*\n(.*?)```", re.S)


# --------------------------------------------------------------------------------------------------
# Candidates
# --------------------------------------------------------------------------------------------------

def hud_candidate():
    root = BENCH.parent
    meta = json.loads(subprocess.run(["cargo", "metadata", "--no-deps", "--format-version", "1"], cwd=root,
                                     capture_output=True, text=True, check=True).stdout)
    version = next(p["version"] for p in meta["packages"] if p["name"] == "hud")
    bins = BENCH / "candidates" / "hud" / "src" / "bin"
    return {
        "name": "hud", "crate": "hud", "version": version, "project": root,
        "dependency": f'hud = {{ path = "{root / "crates" / "hud"}" }}',
        "solutions": {t: bins / (t.split("-")[0] + "_" + t.split("-")[1] + ".rs") for t in ALL_TASKS
                      if (bins / (t.split("-")[0] + "_" + t.split("-")[1] + ".rs")).exists()},
    }


CANDIDATES = {"hud": hud_candidate}


# --------------------------------------------------------------------------------------------------
# Prompt, extraction, literal check, audit
# --------------------------------------------------------------------------------------------------

def prompt_template():
    text = (SPEC / "dx-prompt.md").read_text()
    return re.search(r"```text\n(.*?)\n```", text, re.S).group(1)


def render_prompt(candidate, task_id):
    statement = (SPEC / "tasks" / f"{task_id}.md").read_text().strip()
    return (prompt_template().replace("{crate}", candidate["crate"]).replace("{version}", candidate["version"])
            .replace("{contents of spec/tasks/<task-id>.md}", statement))


def extract_rust(reply):
    blocks = RUST_BLOCK.findall(reply or "")
    return blocks[-1] if blocks else None


def target_lines(task_id):
    """Non-trivial lines of the target blocks embedded in the task statement."""
    text = (SPEC / "tasks" / f"{task_id}.md").read_text()
    lines = []
    for block in re.findall(r"```text\n(.*?)```", text, re.S):
        lines += [ln.strip() for ln in block.splitlines() if len(ln.strip()) >= 12]
    return lines


def string_literals(source):
    out = []
    for m in re.finditer(r'r(#*)"(.*?)"\1|"((?:[^"\\]|\\.)*)"', source, re.S):
        out.append(m.group(2) if m.group(2) is not None else m.group(3))
    return "\n".join(s.replace("\\n", "\n").replace('\\"', '"') for s in out)


def literal_output(source, task_id):
    """True when the program carries most of the target text as string literals (the prompt forbids it)."""
    lines = target_lines(task_id)
    if len(lines) < 2:
        return False
    literals = string_literals(source)
    hit = sum(1 for ln in lines if ln in literals)
    return hit >= max(2, (len(lines) + 1) // 2)


def tool_uses(events):
    for ev in events:
        content = (ev.get("message") or {}).get("content") if isinstance(ev, dict) else None
        for block in content if isinstance(content, list) else []:
            if isinstance(block, dict) and block.get("type") == "tool_use":
                yield block.get("name"), block.get("input") or {}


def audit(events, run_dir, mirror):
    """(docs_read, violations): every tool must be Read, Grep or Glob and every path inside the mirror."""
    mirror = Path(mirror).resolve()
    violations, docs_read = [], False
    for name, args in tool_uses(events):
        if name not in ALLOWED_TOOLS:
            violations.append(f"tool {name}")
            continue
        for key in ("file_path", "path", "pattern"):
            value = args.get(key)
            if not value or not isinstance(value, str):
                continue
            if key == "pattern" and name != "Glob":
                continue
            target = Path(value) if os.path.isabs(value) else Path(run_dir) / value
            if key == "pattern":
                target = Path(str(target).split("*")[0])
            resolved = target.resolve()
            if mirror == resolved or mirror in resolved.parents:
                docs_read = True
            else:
                violations.append(f"{name} {value}")
    return docs_read, violations


# --------------------------------------------------------------------------------------------------
# Agents
# --------------------------------------------------------------------------------------------------

class MockAgent:
    """No model: answers with the candidate's hand-written solution (or a deliberate failure) and a transcript."""

    def __init__(self, candidate, mode="solution"):
        self.candidate, self.mode = candidate, mode

    def run(self, prompt, run_dir, task_id, model, limits):
        events = [{"type": "assistant", "message": {"content": [
            {"type": "tool_use", "name": "Read", "input": {"file_path": str(Path(run_dir) / "README.md")}}]}}]
        source = self.candidate["solutions"].get(task_id)
        source = source.read_text() if source else None
        if self.mode == "peek":
            events.append({"type": "assistant", "message": {"content": [
                {"type": "tool_use", "name": "Read", "input": {"file_path": "/etc/hosts"}}]}})
        if self.mode == "broken" and source:
            source = source.replace("fn main", "fn main_broken_on_purpose(")
        if self.mode == "wrong" and source:
            source = 'fn main() { println!("wrong"); }\n'
        reply = f"```rust\n{source}```" if source else "I cannot do this."
        return {"reply": reply, "events": events, "usage": {"input_tokens": 0, "output_tokens": 0, "cost_usd": 0.0}}


class ClaudeCli:
    """The Claude Code CLI in print mode with Read, Grep and Glob only. Not run by any test: it spends tokens."""

    def __init__(self, isolation="bare"):
        self.isolation = isolation

    def command(self, prompt, run_dir, model, limits):
        cmd = ["claude", "-p", prompt, "--model", model, "--output-format", "stream-json", "--verbose",
               "--tools", "Read,Grep,Glob", "--allowed-tools", "Read", "Grep", "Glob", "--permission-mode", "dontAsk",
               "--no-session-persistence", "--disable-slash-commands", "--strict-mcp-config",
               "--max-turns", str(limits["max_turns"])]
        if limits.get("max_budget_usd_per_run"):
            cmd += ["--max-budget-usd", str(limits["max_budget_usd_per_run"])]
        if self.isolation == "bare":
            cmd.append("--bare")
        return cmd

    def run(self, prompt, run_dir, task_id, model, limits):
        proc = subprocess.run(self.command(prompt, run_dir, model, limits), cwd=run_dir, capture_output=True, text=True,
                              timeout=limits["timeout_s"], stdin=subprocess.DEVNULL)
        events = []
        for line in proc.stdout.splitlines():
            try:
                events.append(json.loads(line))
            except ValueError:
                continue
        result = next((e for e in reversed(events) if e.get("type") == "result"), {})
        usage = result.get("usage") or {}
        return {"reply": result.get("result", ""), "events": events,
                "usage": {"input_tokens": usage.get("input_tokens", 0) + usage.get("cache_read_input_tokens", 0),
                          "output_tokens": usage.get("output_tokens", 0), "cost_usd": result.get("total_cost_usd", 0.0)}}


# --------------------------------------------------------------------------------------------------
# Sandbox, build, check
# --------------------------------------------------------------------------------------------------

def prepare_mirror(candidate, out):
    out = Path(out)
    subprocess.run([PYTHON, str(BENCH / "scripts" / "docs_mirror.py"), str(candidate["project"]), candidate["crate"],
                    str(out)], check=True, capture_output=True, text=True)
    return out


def prepare_sandbox(candidate, out):
    out = Path(out)
    (out / "src").mkdir(parents=True, exist_ok=True)
    (out / "Cargo.toml").write_text(
        '[package]\nname = "dx_sandbox"\nversion = "0.0.0"\nedition = "2024"\npublish = false\n\n[workspace]\n\n'
        f'[dependencies]\n{candidate["dependency"]}\n')
    (out / "src" / "main.rs").write_text("fn main() {}\n")
    subprocess.run(["cargo", "generate-lockfile", "--offline"], cwd=out, check=True, capture_output=True, text=True)
    subprocess.run(["cargo", "build", "--release", "--locked", "--offline"], cwd=out, check=True, capture_output=True,
                   text=True)
    return out


def build(sandbox, source):
    (Path(sandbox) / "src" / "main.rs").write_text(source)
    proc = subprocess.run(["cargo", "build", "--release", "--locked", "--offline", "--color", "never"],
                          cwd=sandbox, capture_output=True, text=True)
    return proc.returncode == 0, (proc.stderr or "")[-2000:], Path(sandbox) / "target" / "release" / "dx_sandbox"


def check(task_id, binary):
    proc = subprocess.run([PYTHON, str(BENCH / "scripts" / "tasks.py"), "check", task_id, str(binary)],
                          capture_output=True, text=True, cwd=BENCH)
    return proc.returncode == 0, proc.stdout[-1500:]


def lines_of_code(path):
    import loc
    return loc.count(Path(path))


# --------------------------------------------------------------------------------------------------
# One run
# --------------------------------------------------------------------------------------------------

def one_run(candidate, agent, model, task_id, repeat, work, mirror, sandbox, limits):
    run_id = f"{model}-{task_id}-{repeat}"
    run_dir = Path(work) / "runs" / run_id
    if run_dir.exists():
        shutil.rmtree(run_dir)
    run_dir.mkdir(parents=True)
    (run_dir / "docs").symlink_to(Path(mirror) / "docs")
    if (Path(mirror) / "README.md").exists():
        (run_dir / "README.md").symlink_to(Path(mirror) / "README.md")
    row = {"candidate": candidate["name"], "model": model, "task": task_id, "repeat": repeat, "status": None,
           "reason": None, "loc": None, "usage": None}
    if task_id not in candidate["solutions"] and isinstance(agent, MockAgent):
        return {**row, "status": "unsupported", "reason": "no hand-written solution for the mock"}
    try:
        got = agent.run(render_prompt(candidate, task_id), run_dir, task_id, model, limits)
    except (subprocess.TimeoutExpired, OSError) as error:
        return {**row, "status": "harness_error", "reason": f"agent: {error}"}
    row["usage"] = got["usage"]
    (run_dir / "transcript.jsonl").write_text("\n".join(json.dumps(e) for e in got["events"]) + "\n")
    docs_read, violations = audit(got["events"], run_dir, Path(mirror))
    if violations:
        return {**row, "status": "protocol_violation", "reason": "; ".join(violations[:3])}
    if not docs_read:
        return {**row, "status": "protocol_violation", "reason": "the docs were not read before the reply"}
    source = extract_rust(got["reply"])
    if source is None:
        return {**row, "status": "candidate_failure", "reason": "no rust block in the reply"}
    (run_dir / "main.rs").write_text(source)
    row["loc"] = lines_of_code(run_dir / "main.rs")
    if literal_output(source, task_id):
        return {**row, "status": "candidate_failure", "reason": "the target text is in string literals"}
    ok, stderr, binary = build(sandbox, source)
    if not ok:
        first = next((ln for ln in stderr.splitlines() if ln.startswith("error") and "could not compile" not in ln), "")
        return {**row, "status": "candidate_failure", "reason": ("compile error: " + first[:160]).strip()}
    passed, output = check(task_id, binary)
    if not passed:
        return {**row, "status": "candidate_failure", "reason": "output differs from the golden"}
    return {**row, "status": "success"}


# --------------------------------------------------------------------------------------------------
# Estimate and report
# --------------------------------------------------------------------------------------------------

def load_config():
    return json.loads((SPEC / "dx-models.json").read_text())


def estimate(config, mirror=None, tasks=None):
    """Tokens (and cost when prices are configured) of a full run. An estimate from stated assumptions, not a measurement."""
    est = config["estimate"]
    readme = page = 0
    if mirror and (Path(mirror) / "docs").exists():
        pages = list((Path(mirror) / "docs").rglob("*.html"))
        page = sum(p.stat().st_size for p in pages) // max(len(pages), 1) // 4
        readme = (Path(mirror) / "README.md").stat().st_size // 4 if (Path(mirror) / "README.md").exists() else 0
    reads = readme + est["assumed_doc_pages_read"] * (page or 3000)
    # Every turn re-sends the context so far: the system context plus the reads gathered up to that turn.
    input_tokens = est["assumed_turns"] * (est["system_context_tokens"] + reads // 2) + reads
    output_tokens = est["output_tokens_per_run"]
    n_tasks = len(tasks or ALL_TASKS)
    plan = [("primary", config["primary"]), ("stress", config["stress"])]
    rows = []
    for name, spec in plan:
        runs = n_tasks * spec["repeats"]
        row = {"role": name, "model": spec["id"], "runs": runs, "input_tokens": runs * input_tokens,
               "output_tokens": runs * output_tokens, "cost_usd": None}
        price = est["prices_usd_per_mtok"].get(spec["id"] or "")
        if price:
            row["cost_usd"] = (row["input_tokens"] * price["input"] + row["output_tokens"] * price["output"]) / 1e6
        rows.append(row)
    return {"per_run": {"input_tokens": input_tokens, "output_tokens": output_tokens}, "plan": rows,
            "assumptions": {**est, "doc_page_tokens": page, "readme_tokens": readme}}


def bootstrap_ci(per_task_rates, seed=20261008, n=BOOTSTRAP):
    rng = random.Random(seed)
    if not per_task_rates:
        return None
    means = sorted(statistics.fmean(rng.choices(per_task_rates, k=len(per_task_rates))) for _ in range(n))
    return [means[int(0.025 * n)], means[int(0.975 * n) - 1]]


def report(rows, threshold=0.8, min_runs=40):
    """First-try success per model: counted runs exclude harness_error, protocol_violation and unsupported."""
    out = {}
    for model in sorted({r["model"] for r in rows}):
        mine = [r for r in rows if r["model"] == model and r["status"] in ("success", "candidate_failure")]
        by_task = {}
        for r in mine:
            by_task.setdefault(r["task"], []).append(r)
        rates = {t: sum(r["status"] == "success" for r in rs) / len(rs) for t, rs in by_task.items()}
        loc = {t: statistics.median(r["loc"] for r in rs if r["loc"]) for t, rs in by_task.items()
               if any(r["loc"] for r in rs)}
        runs = len(mine)
        pipeline_check = runs < min_runs or model.startswith("mock-")
        out[model] = {
            "runs": runs, "success": sum(r["status"] == "success" for r in mine),
            "first_try": (sum(r["status"] == "success" for r in mine) / runs) if runs else None,
            "per_task": rates, "median_loc_per_task": loc,
            "ci95_over_tasks": bootstrap_ci(list(rates.values())),
            "excluded": {s: sum(r["status"] == s for r in rows if r["model"] == model)
                         for s in ("harness_error", "protocol_violation", "unsupported")},
            "pipeline_check": pipeline_check,
            "gate": "inconclusive (pipeline check)" if pipeline_check else (
                "pass" if sum(r["status"] == "success" for r in mine) / runs >= threshold else "fail"),
        }
    return out


# --------------------------------------------------------------------------------------------------
# Commands
# --------------------------------------------------------------------------------------------------

def cmd_estimate(args):
    config = load_config()
    candidate = CANDIDATES[args.candidate]()
    mirror = Path(args.work) / "mirror"
    if not mirror.exists():
        prepare_mirror(candidate, mirror)
    print(json.dumps(estimate(config, mirror, args.tasks), indent=1))


def cmd_run(args):
    config = load_config()
    candidate = CANDIDATES[args.candidate]()
    tasks = args.tasks or ALL_TASKS
    limits = config["limits"]
    work = Path(args.work)
    work.mkdir(parents=True, exist_ok=True)
    if args.execute and args.dry_run:
        sys.exit("--execute and --dry-run exclude each other")
    if not args.execute and not args.dry_run:
        sys.exit("choose --dry-run (mock agent, no model) or --execute (real agent, spends tokens)")
    primary = args.primary_model or config["primary"]["id"]
    stress = args.stress_model or config["stress"]["id"]
    repeats_primary = args.repeats if args.repeats is not None else config["primary"]["repeats"]
    repeats_stress = args.stress_repeats if args.stress_repeats is not None else (
        args.repeats if args.repeats is not None else config["stress"]["repeats"])
    if args.dry_run:
        agent = MockAgent(candidate, args.mock)
        plan = [("mock-primary", repeats_primary), ("mock-stress", repeats_stress)]
    else:
        if not primary or not stress:
            sys.exit("a real run needs both model ids: set them in spec/dx-models.json or pass --primary-model and --stress-model")
        agent = ClaudeCli(config["isolation"])
        plan = [(primary, repeats_primary), (stress, repeats_stress)]
    plan = [(model, repeats) for model, repeats in plan if repeats > 0]
    mirror = prepare_mirror(candidate, work / "mirror")
    sandbox = prepare_sandbox(candidate, work / "sandbox")
    est = estimate(config, mirror, tasks)
    print("estimate (assumptions in spec/dx-models.json):", json.dumps(est["plan"]), file=sys.stderr)
    if args.execute and not args.yes:
        if input("run the real agent? [y/N] ").strip().lower() != "y":
            sys.exit("not confirmed")
    rows = []
    started = time.time()
    for model, repeats in plan:
        for task in tasks:
            for repeat in range(1, repeats + 1):
                rows.append(one_run(candidate, agent, model, task, repeat, work, mirror, sandbox, limits))
                print(f"{model} {task} #{repeat}: {rows[-1]['status']}" + (f" ({rows[-1]['reason']})" if rows[-1]["reason"] else ""),
                      file=sys.stderr)
    out = Path(args.out) if args.out else work / "runs.jsonl"
    out.write_text("\n".join(json.dumps(r) for r in rows) + "\n")
    summary = report(rows, config["threshold_first_try"])
    print(json.dumps({"runs_file": str(out), "seconds": round(time.time() - started, 1), "report": summary}, indent=1))


def cmd_report(args):
    rows = [json.loads(line) for line in Path(args.runs).read_text().splitlines() if line.strip()]
    print(json.dumps(report(rows, load_config()["threshold_first_try"]), indent=1))


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    for name in ("estimate", "run"):
        p = sub.add_parser(name)
        p.add_argument("--candidate", default="hud", choices=sorted(CANDIDATES))
        p.add_argument("--tasks", nargs="+", choices=ALL_TASKS)
        p.add_argument("--work", default=str(BENCH / "results" / "dx"))
    run = sub.choices["run"]
    run.add_argument("--dry-run", action="store_true")
    run.add_argument("--execute", action="store_true")
    run.add_argument("--yes", action="store_true")
    run.add_argument("--mock", default="solution", choices=["solution", "broken", "wrong", "peek"])
    run.add_argument("--repeats", type=int)
    run.add_argument("--stress-repeats", type=int)
    run.add_argument("--primary-model")
    run.add_argument("--stress-model")
    run.add_argument("--out")
    rep = sub.add_parser("report")
    rep.add_argument("runs")
    args = ap.parse_args()
    {"estimate": cmd_estimate, "run": cmd_run, "report": cmd_report}[args.cmd](args)


if __name__ == "__main__":
    main()
